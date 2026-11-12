//! Page-side synchronization for compositor and main-thread wheel routes.
//! Interaction calculates main-thread updates from the resident fragments;
//! Page applies their typed mutations and routes compositor continuations.
use super::*;
use interaction::input_event::{WheelEvent, WheelPhase};

impl PageState {
    /// ChromeClientImpl::InjectScrollbarGestureScroll, cc:442-449, and
    /// WebFrameWidgetImpl::InjectScrollbarGestureScroll, cc:4326-4355.
    ///
    /// Interaction has already selected the ScrollableArea and applied the
    /// local-root input scale. The generated event retains that opaque scroll
    /// identity and enters the same sequenced compositor ingress as wheel
    /// continuations. The queue is drained after synchronous input dispatch;
    /// neither this callback nor a scrollbar timer mutates DOM scroll offsets.
    pub(super) fn InjectScrollbarGesture(
        &self,
        request: interaction::scroll::core::scroll::scrollbar::ScrollbarGesture,
    ) -> io::Result<()> {
        use compositor::adapter::input_types::{ElementId, ScrollGranularity};
        use interaction::input::{
            base::time::time::TimeTicks,
            public::common::input::{web_gesture_event::WebGestureEvent, web_input_event::Type},
            ui::events::types::scroll_types::ScrollGranularity as SourceGranularity,
        };

        let element = request.scroll_element_id.GetInternalValue();
        if !ElementId::IsValidInternalValue(element)
            || !request.delta.x().is_finite()
            || !request.delta.y().is_finite()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid injected scrollbar gesture",
            ));
        }
        let phase = match request.gesture_type {
            Type::kGestureScrollBegin => ScrollPhase::Begin,
            Type::kGestureScrollUpdate => ScrollPhase::Update,
            Type::kGestureScrollEnd => ScrollPhase::End,
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "scrollbar injection requires a scroll gesture",
                ));
            }
        };
        // The external compositor supplies its own absolute clock domain.
        // Local pipelines use the identical mapping owned by their executor.
        let now = std::time::Instant::now();
        let timestamp = self.external_compositor_clock.borrow().map_or_else(
            || self.rendering.borrow().executor.TimeTicks(now),
            |mapping| mapping.TimeTicks(now),
        );
        let mut gesture = WebGestureEvent::GenerateInjectedScrollbarGestureScroll(
            request.gesture_type,
            TimeTicks::FromInternalValue(timestamp.since_origin().InMicroseconds()),
            foundation::gfx::PointF::default(),
            request.delta,
            request.granularity,
        );
        if phase == ScrollPhase::Begin {
            gesture.data.scroll_begin.scrollable_area_element_id = element;
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
            ScrollPhase::End => Vector2dF::default(),
        };
        let granularity = match request.granularity {
            SourceGranularity::kScrollByPrecisePixel => ScrollGranularity::kScrollByPrecisePixel,
            SourceGranularity::kScrollByPixel => ScrollGranularity::kScrollByPixel,
            SourceGranularity::kScrollByLine => ScrollGranularity::kScrollByLine,
            SourceGranularity::kScrollByPage => ScrollGranularity::kScrollByPage,
            SourceGranularity::kScrollByDocument => ScrollGranularity::kScrollByDocument,
        };
        self.pending_compositor_mutations
            .borrow_mut()
            .push(PageBrowserInput::Compositor(
                compositor::CompositorMutation::Input(CompositorInput::ScrollbarGesture {
                    scroll_element_id: ElementId::new(element),
                    delta,
                    granularity,
                    event_timestamp: timestamp,
                    phase,
                }),
            ));
        Ok(())
    }
}

pub(super) enum WidgetPointQuery {
    ScrollContainer,
    BlockBounds,
}
pub(super) enum WidgetPointResult {
    ScrollContainer(Option<foundation::CompositorElementId>),
    BlockBounds(foundation::gfx::Rect),
}

impl Page {
    fn EnsureInternalWheelCompositor(&mut self) -> io::Result<()> {
        if self.external_compositor {
            return Ok(());
        }
        let artifact = self
            .frame
            .as_ref()
            .ok_or_else(|| io::Error::other("wheel input requires a committed Page frame"))?
            .display_items
            .clone();
        let constraints = self.state.constraints.borrow();
        let scale = constraints.device_pixel_ratio;
        let physical = |value: f64, name: &str| -> io::Result<u32> {
            let value = value * scale;
            if !value.is_finite() || value <= 0.0 || value > u32::MAX as f64 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("invalid {name} for wheel compositor"),
                ));
            }
            Ok(value.round() as u32)
        };
        let width = physical(constraints.available_size.width, "viewport width")?;
        let height = physical(constraints.available_size.height, "viewport height")?;
        drop(constraints);
        // An internal/headless Page has no host presentation loop to perform
        // this commit. Install the already-published immutable PaintArtifact
        // on the Page-owned compositor before input; repeated commits of the
        // same artifact are admitted by PageRendering's presentable fast path.
        self.state
            .rendering
            .borrow_mut()
            .CommitArtifact(&artifact, width, height, scale)
    }
    fn ScrollableContainerAt(
        &mut self,
        point: Offset,
    ) -> io::Result<Option<compositor::adapter::input_types::ElementId>> {
        // WidgetInputHandlerManager runs this main-thread hit-test only after
        // UpdateLifecycle.  A newly committed navigation can already retain a
        // LayoutResult while its fragment-local PrePaint state is not yet
        // current; UpdateFrameIfNeeded alone would treat a clean Page as a
        // no-op and lend that incomplete owner to Interaction.  Use the same
        // synchronous lifecycle barrier as ordinary EventHandler input before
        // either the child-frame or local-frame hit-test borrows fragments.
        self.CompleteLifecycleForImmediateInput()?;
        let domain = self.state.local_frames.get().unwrap();
        if let Some((child, node)) = domain.HitInputTarget(point)? {
            let runtime = child
                .runtime
                .borrow()
                .clone()
                .ok_or_else(|| io::Error::other("detached child scroll owner"))?;
            let engine = domain
                .FocusInteraction(child.state.Identity())
                .ok_or_else(|| io::Error::other("child scroll input owner required"))?;
            runtime.EnsureInteractionScrollOwners(&child, &engine);
            return Ok(runtime.ScrollableContainerForNode(node).map(|id| {
                compositor::adapter::input_types::ElementId::new(id.GetInternalValue())
            }));
        }
        let WidgetPointResult::ScrollContainer(id) =
            self.WidgetPointAt(point, WidgetPointQuery::ScrollContainer)?
        else {
            unreachable!()
        };
        Ok(id.map(|id| compositor::adapter::input_types::ElementId::new(id.GetInternalValue())))
    }
    /// Route the source widget query to Interaction's borrowed resident owners.
    pub(super) fn WidgetPointAt(
        &mut self,
        point: Offset,
        kind: WidgetPointQuery,
    ) -> io::Result<WidgetPointResult> {
        use interaction::adapter::page_cursor::{
            PreparedResidentCursorQuery, QueryPreparedResidentScrollContainer,
            ResidentCursorHostMetrics,
        };
        // WidgetInputHandlerManager's hit-test task updates Blink's lifecycle
        // before borrowing the native hit result and returns only the id.
        self.CompleteLifecycleForImmediateInput()?;
        self.EnsureInteractionScrollOwners();
        let frame = self
            .frame
            .as_ref()
            .ok_or_else(|| io::Error::other("scroll hit test requires a frame"))?;
        let layout = self.state.layout_engine.borrow();
        let output = layout
            .Output()
            .ok_or_else(|| io::Error::other("scroll hit test requires layout"))?;
        let paint = self.state.paint_engine.borrow();
        let owners = self
            .interaction_scroll_owners
            .as_ref()
            .ok_or_else(|| io::Error::other("scroll hit test requires scroll owner"))?;
        let scroll = owners.ScrollManager();
        let mut image_failures = Vec::new();
        let query = PreparedResidentCursorQuery {
            layout: output,
            fragments: PaintFragmentCursor::FromHandle(&frame.fragments),
            paint: &paint,
            scroll: &scroll,
            paths: &paint_skia::path_hit_test::SkiaPathHitTestFactory,
            selections: &self.state.layout_editing.selections,
            lifecycle_clean: true,
            child_frames: &super::local_frame::LocalFrameQueries {
                domain: self.state.local_frames.get().unwrap(),
                paint: &paint,
            },
            native_paths: Arc::new(paint_skia::svg_shape_path::SkiaNativePathFactory),
            editing_positions: Rc::new(resident_editing_positions::PageResidentEditingPositions(
                Rc::downgrade(&self.state),
            )),
            image_failures: &mut image_failures,
            // The read-only hit query never enters cursor selection.
            host_metrics: ResidentCursorHostMetrics {
                device_scale_factor: self.state.constraints.borrow().device_pixel_ratio as f32,
                system_cursor_size_dip: self
                    .state
                    .native_event_handler_host
                    .system_cursor_size_dip
                    .get()
                    .ok_or_else(|| io::Error::other("native system cursor metrics required"))?,
            },
            preferred_color_scheme:
                interaction::core::input::event_handler_cursor::ColorScheme::kLight,
        };
        match kind {
            WidgetPointQuery::ScrollContainer => QueryPreparedResidentScrollContainer(
                &self.interaction,
                &self.state.document.Handle(),
                point,
                query,
            )
            .map(WidgetPointResult::ScrollContainer),
            WidgetPointQuery::BlockBounds => {
                interaction::adapter::page_cursor::QueryPreparedResidentBlockBounds(
                    &self.interaction,
                    &self.state.document.Handle(),
                    point,
                    query,
                )
                .map(WidgetPointResult::BlockBounds)
            }
        }
        .map_err(|error| io::Error::other(format!("widget hit test: {error:?}")))
    }
    fn CommitCompositorScrollOffsets(&mut self) {
        self.state
            .local_frames
            .get()
            .unwrap()
            .CommitCompositorScrollOffsets();
    }
    fn IsVisualViewportScrollElement(
        &self,
        element: compositor::adapter::input_types::ElementId,
    ) -> bool {
        self.frame
            .as_ref()
            .and_then(|frame| frame.display_items.compositor_viewport.as_ref())
            .and_then(|viewport| viewport.visual_scroll_translation.scroll.as_deref())
            .is_some_and(|scroll| {
                scroll.compositor.compositor_element_id.GetInternalValue()
                    == element.GetInternalValue()
            })
    }
    fn ApplyCompositorScrollOffset(
        &mut self,
        sequence: u64,
        element: compositor::adapter::input_types::ElementId,
        offset: Offset,
        source_type: compositor::ScrollSourceType,
    ) -> io::Result<Option<u64>> {
        if sequence <= self.processed_compositor_scroll_sequence {
            return Ok(None);
        }
        if !offset.x.is_finite() || !offset.y.is_finite() {
            return Err(io::Error::other("nonfinite compositor scroll offset"));
        }
        if !self.state.document.HasRenderableRoot() {
            self.processed_compositor_scroll_sequence = sequence;
            return Ok(None);
        }
        // ScrollingCoordinator reflects an impl-thread offset against the
        // already committed layout state.  It must not enter the document
        // lifecycle from input dispatch: a dirty style tree is serviced by
        // the next normal lifecycle update.
        let native_element =
            foundation::CompositorElementId::new(element.GetInternalValue());
        // Paint's resident scroll nodes and cc's scroll tree share Layout's
        // zoomed coordinate space. InputRouter scales pixel wheel deltas into
        // that space before cc consumes them, so the absolute impl offset is
        // already the value ScrollingCoordinator must reflect. Multiplying it
        // again here made every main-thread commit reset the active tree to a
        // different offset, observed as a jump or rebound at DPR != 1.
        let main = self
            .state
            .layout_engine
            .borrow_mut()
            .DidCompositorScroll(native_element, offset, source_type);
        let Some(update) = main else {
            let target = self
                .state
                .local_frames
                .get()
                .unwrap()
                .ReflectCompositorScrollElement(native_element, offset, source_type);
            self.processed_compositor_scroll_sequence = sequence;
            return Ok(target);
        };
        self.state.ApplyMutation(PageMutation::ScrollMutation(
            page_mutation::ScrollMutation {
                target_node_id: update.target_node_id,
                offset: update.offset,
                scroll_type:
                    layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kCompositor,
            },
        ));
        // DidCompositorScroll has already committed the resident native area.
        // The DOM may also already expose this value (for example after the
        // input transaction reflected its logical offset), in which case the
        // ordinary Document write is intentionally a no-op. ScrollingCoordinator
        // must still publish the compositor acknowledgement to the retained
        // fragment/property state, so enqueue that typed lifecycle mutation
        // independently of whether the scalar DOM value changed.
        {
            let mut pending = self.state.scroll_updates.borrow_mut();
            if let Some((_, offset, scroll_type)) = pending
                .iter_mut()
                .find(|(target, _, _)| *target == update.target_node_id)
            {
                *offset = update.offset;
                *scroll_type =
                    layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kCompositor;
            } else {
                pending.push((
                    update.target_node_id,
                    update.offset,
                    layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kCompositor,
                ));
            }
        }
        self.state.compositor_scroll_updates.set(true);
        self.state.InvalidateLayoutOutput();
        if !self.state.dirty.get() {
            self.state.scroll_only.set(true);
        }
        self.state.dirty.set(true);
        self.processed_compositor_scroll_sequence = sequence;
        Ok(Some(update.target_node_id))
    }
    pub fn ApplyCompositorWheelEffects(
        &mut self,
        effects: Vec<CompositorMainThreadEffect>,
    ) -> io::Result<()> {
        let mut requested_main_frame = false;
        for effect in effects {
            if std::env::var_os("BROWSER_PROFILE_INPUT").is_some() {
                eprintln!("page-compositor-main-effect {effect:?}");
            }
            match effect {
                CompositorMainThreadEffect::MainFrameCommitCompleted { commit_finish_time } => {
                    let time = self
                        .state
                        .rendering
                        .borrow()
                        .PresentationClockMapping()
                        .Instant(commit_finish_time);
                    if let Some(time) = time {
                        self.EventTimingDidCommit(time);
                    }
                }
                CompositorMainThreadEffect::ScrollAnimationCommitPrepared { .. }
                | CompositorMainThreadEffect::ScrollAnimationCommitFinalized { .. }
                | CompositorMainThreadEffect::ScrollAnimationCompleted(_) => {
                    return Err(io::Error::other(
                        "compositor scroll animation requires an interaction animation owner",
                    ));
                }
                CompositorMainThreadEffect::WheelRequiresMainThreadHitTest {
                    request,
                    position,
                } => {
                    // InputRouter hands cc a root-widget point after applying
                    // the layout zoom. WidgetInputHandlerManager performs its
                    // fallback hit test in Blink's CSS viewport coordinates,
                    // so cross back through the same current active-artifact
                    // scale before borrowing Interaction's resident cursor.
                    let widget_scale =
                        f64::from(self.state.rendering.borrow().InputWidgetScale());
                    let scroll_element_id = self.ScrollableContainerAt(Offset {
                        x: f64::from(position.x()) / widget_scale,
                        y: f64::from(position.y()) / widget_scale,
                    })?;
                    self.RouteWheelCompositorInput(CompositorInput::ContinueWheelScroll {
                        request,
                        scroll_element_id,
                    })?;
                }
                CompositorMainThreadEffect::WheelRequiresDOMAcknowledgement { .. } => {
                    return Err(io::Error::other(
                        "wheel acknowledgement must retain its original host event",
                    ));
                }
                CompositorMainThreadEffect::ViewportScaleChanged {
                    delta,
                    inner_scroll_delta,
                    ..
                } => {
                    {
                        let mut zoom = self.state.web_view_zoom.borrow_mut();
                        zoom.ApplyScaleDelta(delta);
                        zoom.ApplyVisualOffsetDelta(foundation::gfx::Vector2dF::new(
                            inner_scroll_delta.x(),
                            inner_scroll_delta.y(),
                        ));
                    }
                    self.state.SyncVisualViewportPaintState();
                    self.state.dirty.set(true);
                    requested_main_frame = true;
                }
                CompositorMainThreadEffect::PageScaleAnimationCompleted => {
                    self.state
                        .web_view_zoom
                        .borrow_mut()
                        .DidCompletePageScaleAnimation();
                }
                CompositorMainThreadEffect::CanvasChildPaintRecords(records) => {
                    self.state
                        .document
                        .borrow()
                        .GetDocument()
                        .PublishCanvasChildRecords(records);
                }
                CompositorMainThreadEffect::BeginMainFrame
                | CompositorMainThreadEffect::RequestMainFrame => requested_main_frame = true,
                CompositorMainThreadEffect::UpdateCompositorScrollState(scroll) => {
                    let gesture_ended = self.wheel_scroll_active && !scroll.is_scroll_active;
                    self.wheel_scroll_active = scroll.is_scroll_active;
                    if gesture_ended || !scroll.done_containers.is_empty() {
                        // The final native offsets precede completion. Active
                        // commits retain their ordinary compositor-only route.
                        // Do not consume the root's typed pending update here:
                        // this callback is still inside the compositor-effect
                        // batch. The enclosing UpdateFrameIfNeeded transaction
                        // publishes it with the retained PageFrame after the
                        // batch releases its compositor owners.
                        self.CommitCompositorScrollOffsets();
                    }
                    if gesture_ended {
                        self.hover_state_dirty = self.cursor_position.is_some();
                    }
                }
                CompositorMainThreadEffect::ScrollOffsetChanged {
                    sequence,
                    element,
                    offset,
                    source,
                    ..
                } => {
                    // ProxyImpl reports the inner viewport delta separately in
                    // ViewportScaleChanged. Its PAC callback carries the same
                    // synthetic VisualViewport element, which deliberately has
                    // no DOM/Layout owner. Acknowledge that callback here after
                    // the typed viewport effect has entered this batch; do not
                    // reflect it through ScrollingCoordinator's DOM path.
                    if self.IsVisualViewportScrollElement(element) {
                        self.processed_compositor_scroll_sequence =
                            self.processed_compositor_scroll_sequence.max(sequence);
                        browser_tracing::instant(
                            "input",
                            "Page.VisualViewportScrollOffsetApplied",
                            &[
                                ("sequence", sequence as f64),
                                ("element", element.GetInternalValue() as f64),
                                ("offset_x", f64::from(offset.x())),
                                ("offset_y", f64::from(offset.y())),
                            ],
                        );
                        continue;
                    }
                    let target = self.ApplyCompositorScrollOffset(
                        sequence,
                        element,
                        Offset {
                            x: f64::from(offset.x()),
                            y: f64::from(offset.y()),
                        },
                        source,
                    )?;
                    let Some(target) = target else {
                        browser_tracing::instant(
                            "input",
                            "Page.CompositorScrollOwnerMissing",
                            &[
                                ("sequence", sequence as f64),
                                ("element", element.GetInternalValue() as f64),
                            ],
                        );
                        continue;
                    };
                    browser_tracing::instant(
                        "input",
                        "Page.CompositorScrollOffsetApplied",
                        &[
                            ("sequence", sequence as f64),
                            ("element", element.GetInternalValue() as f64),
                            ("target", target as f64),
                        ],
                    );
                }
                // Source: core/page/scrolling/scrolling_coordinator.cc:88-100.
                // Element-id lookup is shared with compositor scroll offsets;
                // the native PaintLayerScrollableArea retains the visibility.
                CompositorMainThreadEffect::ScrollbarsHiddenChanged { element, hidden } => {
                    let native_element =
                        foundation::CompositorElementId::new(element.GetInternalValue());
                    if self
                        .state
                        .local_frames
                        .get()
                        .unwrap()
                        .UpdateCompositorScrollbarsHiddenElement(native_element, hidden)
                    {
                        requested_main_frame = true;
                        continue;
                    }
                    let target = self
                        .state
                        .layout_engine
                        .borrow()
                        .CompositorScrollArea(native_element)
                        .map(|area| area.target_node_id);
                    if target.is_some_and(|target| {
                        self.state
                            .layout_engine
                            .borrow_mut()
                            .UpdateScrollbarsHiddenIfOverlay(target, hidden)
                    })
                    {
                        self.state.InvalidateLayoutOutput();
                        self.state.scroll_only.set(true);
                        self.state.dirty.set(true);
                        requested_main_frame = true;
                    }
                }
                _ => {}
            }
        }
        // Source: ProxyMain::BeginMainFrame. The applied compositor delta and
        // the property-tree commit belong to one main-frame transaction.
        if requested_main_frame {
            self.CommitCompositorScrollOffsets();
        }
        Ok(())
    }
    pub(super) fn PresentCompositorOnlyFrame(&mut self) {
        let Some(frame) = self.frame.as_mut() else {
            return;
        };
        frame.sequence = frame.sequence.wrapping_add(1).max(1);
        frame.compositor_scroll_sequence = self.processed_compositor_scroll_sequence;
        self.state.client.borrow_mut().DidPresentFrame(frame);
    }

    /// Apply cc's scroll-and-scale delta batch before the main rendering
    /// opportunity. The host verifies document identity; Page owns DOM lookup
    /// and publishes this acknowledgement only with a completed paint frame.
    pub fn ApplyCompositorScrollDelta(
        &mut self,
        sequence: u64,
        owner_client_id: u64,
        delta: Offset,
    ) -> io::Result<()> {
        if sequence <= self.processed_compositor_scroll_sequence {
            return Ok(());
        }
        if !delta.x.is_finite() || !delta.y.is_finite() {
            return Err(io::Error::other("nonfinite compositor scroll delta"));
        }
        // ScrollingCoordinator ignores callbacks for deleted scroll owners.
        // Never force measurement before a replacement document has a root.
        if !self.state.document.HasRenderableRoot() {
            self.processed_compositor_scroll_sequence = sequence;
            return Ok(());
        }
        // Consume the resident committed output.  Pulling layout forward here
        // would put style recalculation and layout on the wheel dispatch path.
        let range = self
            .state
            .layout_engine
            .borrow()
            .Output()
            .map(|output| output.Root())
            .filter(|node| node.PaintClientId() == owner_client_id)
            .map(|node| node.ScrollRange());
        let Some(range) = range else {
            self.processed_compositor_scroll_sequence = sequence;
            return Ok(());
        };
        let (target_node_id, accumulated) = {
            let owner = self.state.document.borrow();
            let document = owner.GetDocument();
            let root = document
                .Node(document.Root())
                .Children()
                .iter()
                .copied()
                .find(|&index| document.Node(index).IsHTMLElement("html"))
                .ok_or_else(|| io::Error::other("compositor scroll has no document root"))?;
            let current = document.ScrollOffsetFor(root);
            (
                document.Node(root).Id(),
                Offset {
                    x: current.x + delta.x,
                    y: current.y + delta.y,
                },
            )
        };
        // Blink reflects the impl batch through its real scroll range. The
        // next delta-bearing main frame waits for this commit to activate,
        // matching the non-overlapping cc main/pending scheduler branch.
        let offset = Offset {
            x: accumulated.x.clamp(range.minimum.x, range.maximum.x),
            y: accumulated.y.clamp(range.minimum.y, range.maximum.y),
        };
        let mutation = PageMutation::ScrollMutation(page_mutation::ScrollMutation {
            target_node_id,
            offset,
            scroll_type: layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kCompositor,
        });
        if let Some(scripts) = &mut self.scripts {
            scripts.ApplyMutation(mutation)?;
        } else {
            self.state.ApplyMutation(mutation);
        }
        self.processed_compositor_scroll_sequence = sequence;
        self.RequestBeginFrameIfNeeded();
        Ok(())
    }

    /// Explicit default-action entry: cancellation was already resolved by the
    /// caller. Normal input uses the original DOM acknowledgement tokens.
    pub fn ScrollWheelDefault(&mut self, wheel: &WheelEvent) -> io::Result<()> {
        // EventHandler enters wheel default handling only after the current
        // document lifecycle has a committed hit-test/compositor snapshot.
        // This public explicit-default entry can follow a synchronous script
        // mutation directly (for example requestFullscreen), so establish the
        // same barrier before asking the compositor to route the wheel. Never
        // fall back to a Page-owned scroll calculation when that owner is
        // temporarily absent.
        self.CompleteLifecycleForImmediateInput()?;
        self.EnsureInternalWheelCompositor()?;
        self.ApplyWheelDefaultWithoutLifecycle(wheel, wheel.target_node_id)?;
        self.CompleteInternalCompositorInputTransaction()?;
        self.UpdateFrameIfNeeded()
    }
    fn CompleteInternalCompositorInputTransaction(&mut self) -> io::Result<()> {
        if self.external_compositor {
            return Ok(());
        }
        // The native embedding posts PageRenderPipeline::Pump to the Page's
        // impl sequence whenever cc leaves an immediately runnable owner turn.
        // The internal/offscreen presenter has no separate impl task runner,
        // so the explicit synchronous default-action entry must execute those
        // same typed turn before returning.  In particular, the input frame
        // may stop at kWaitingOnMain and publish ScrollOffsetChanged from the
        // following owner turn.  Match the native runtime's one posted
        // ImplCommand::Pump for that wake, then run the Page lifecycle requested
        // by its effects.  Do not try to drain cc to quiescence here: after
        // BeginMainFrame/commit it can legitimately be waiting for the next
        // BeginFrame.  Drive the existing compositor scheduler and lifecycle;
        // never calculate a replacement scroll in Page.
        //
        // Source: content/renderer/input/input_handler_proxy.cc;
        //         cc/trees/proxy_impl.cc ProcessScheduledActions.
        if self.state.rendering.borrow().NeedsPump() {
            let result = self.state.rendering.borrow_mut().Pump()?;
            self.ApplyCompositorWheelEffects(result.effects)?;
            if result.submitted {
                self.PresentCompositorOnlyFrame();
            }
            self.UpdateFrameIfNeeded()?;
        }
        Ok(())
    }
    pub(super) fn ApplyWheelDefaultWithoutLifecycle(
        &mut self,
        wheel: &WheelEvent,
        target_node_id: Option<u64>,
    ) -> io::Result<()> {
        // User scrolling interrupts ProgrammaticScrollAnimator in Chromium.
        self.state.ProgrammaticAnimations().CancelAll();
        let ending_phase = matches!(wheel.phase, WheelPhase::kEnded | WheelPhase::kCancelled);
        if wheel.compositor_owned {
            // InputHandler already moved the impl tree. Main receives its
            // actual (clamped) delta at BeginMainFrame, including corrections.
            if ending_phase {
                self.CommitCompositorScrollOffsets();
                if std::mem::take(&mut self.wheel_scroll_active) {
                    self.hover_state_dirty = self.cursor_position.is_some();
                }
            } else if wheel.delta.x != 0.0 || wheel.delta.y != 0.0 {
                self.wheel_scroll_active = true;
            }
            return Ok(());
        }
        if !wheel.compositor_ack.is_empty() {
            for acknowledgement in ::interaction::WheelDOMAcknowledgements(wheel, false) {
                self.RouteWheelCompositorInput(CompositorInput::WheelDOMAcknowledgement {
                    request: acknowledgement.request,
                    default_prevented: acknowledgement.default_prevented,
                })?;
            }
            return Ok(());
        }
        if self.external_compositor {
            return Err(io::Error::other(
                "external wheel default requires its original compositor acknowledgement",
            ));
        }
        let result = self.state.rendering.borrow_mut().Wheel(wheel)?;
        if let Some(request) = result.dom_ack {
            self.RouteWheelCompositorInput(CompositorInput::WheelDOMAcknowledgement {
                request,
                default_prevented: false,
            })?;
        }
        self.ApplyCompositorWheelEffects(result.effects)?;
        if result.submitted {
            self.PresentCompositorOnlyFrame();
        }
        self.wheel_scroll_active |= result.compositor_owned;
        if ending_phase && std::mem::take(&mut self.wheel_scroll_active) {
            self.CommitCompositorScrollOffsets();
            self.hover_state_dirty = self.cursor_position.is_some();
        }
        Ok(())
    }
    pub(super) fn RouteWheelCompositorInput(&mut self, input: CompositorInput) -> io::Result<()> {
        self.RouteCompositorMutation(compositor::CompositorMutation::Input(input))
    }
    pub(super) fn RouteCompositorMutation(
        &mut self,
        mutation: compositor::CompositorMutation,
    ) -> io::Result<()> {
        if self.external_compositor {
            self.state
                .pending_compositor_mutations
                .borrow_mut()
                .push(PageBrowserInput::Compositor(mutation));
            return Ok(());
        }
        let result = self
            .state
            .rendering
            .borrow_mut()
            .ContinueCompositorMutation(mutation)?;
        assert!(
            result.dom_ack.is_none(),
            "continuation cannot dispatch the original DOM wheel twice"
        );
        self.ApplyCompositorWheelEffects(result.effects)?;
        if result.submitted {
            self.PresentCompositorOnlyFrame();
        }
        Ok(())
    }
    pub fn TakePendingBrowserInputs(&self) -> Vec<PageBrowserInput> {
        std::mem::take(&mut *self.state.pending_compositor_mutations.borrow_mut())
    }
}
