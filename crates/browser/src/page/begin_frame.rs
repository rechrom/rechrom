//! Page's owner-thread BeginFrame entry. The embedder produces frame times;
//! interaction owns coalescing, and Page owns dispatch and lifecycle ordering.
use super::*;
use foundation::begin_frame::{BeginFrameArgs, BeginFrameSource};
use interaction::{frame_aligned_input_queue::FrameAlignedInputQueue, input_event::InputEvent};
use std::time::Instant;

#[derive(Default)]
pub(super) struct PageBeginFrame {
    pub(super) source: Option<Arc<dyn BeginFrameSource>>,
    inputs: FrameAlignedInputQueue,
    dispatched: Vec<DispatchedFrameInput>,
    requested: bool,
    executing: bool,
    last_frame: Option<(u64, u64)>,
}
impl PageBeginFrame {
    pub(super) fn InputQueue(&self) -> FrameAlignedInputQueue {
        self.inputs.clone()
    }
    pub(super) fn DidDispatchImmediateInput(&self, input: &InputEvent) {
        self.inputs.DidDispatchImmediateInput(input);
    }
    pub(super) fn RequestUnbufferedInputEvents(&self) {
        self.inputs.RequestUnbufferedInputEvents();
    }
    pub(super) fn WithInputClock(
        clock: Arc<dyn interaction::adapter::host_event_clock::HostInputClock>,
    ) -> Self {
        let state = Self::default();
        state.inputs.SetClock(clock);
        state
    }
}
impl Page {
    pub fn InputDocumentGeneration(&self) -> u64 {
        self.begin_frame.inputs.InputDocumentGeneration()
    }
    pub fn TakeWheelDispositions(
        &mut self,
    ) -> io::Result<Vec<interaction::frame_aligned_input_queue::WheelDisposition>> {
        let dispositions = self.begin_frame.inputs.TakeWheelDispositions();
        for disposition in &dispositions {
            for &request in &disposition.requests {
                self.RouteWheelCompositorInput(CompositorInput::WheelDOMAcknowledgement{
                    request,default_prevented:disposition.result==interaction::input::public::mojom::input::input_event_result::InputEventResultState::kConsumed,
                })?;
            }
        }
        Ok(dispositions)
    }
    pub fn DiscardWheelForDetachedDocument(
        &mut self,
        wheel: &interaction::input_event::WheelEvent,
    ) -> io::Result<()> {
        for acknowledgement in interaction::WheelDOMAcknowledgements(wheel, false) {
            self.RouteWheelCompositorInput(CompositorInput::WheelDOMAcknowledgement {
                request: acknowledgement.request,
                default_prevented: false,
            })?;
        }
        Ok(())
    }
    pub fn GestureTouchActionFilterHandle(&self) -> interaction::input::ui::events::gesture_detection::filtered_gesture_provider::GestureTouchActionFilterHandle{
        self.begin_frame.inputs.TouchActionFilterHandle()
    }
    pub(super) fn NextGestureDeadline(&self, now: Instant) -> Option<Instant> {
        self.begin_frame
            .inputs
            .NextGestureDeadlineDelay()
            .and_then(|delay| now.checked_add(delay))
    }
    pub(super) fn RunGestureTimeouts(&mut self) -> io::Result<()> {
        let inputs = self.begin_frame.inputs.clone();
        inputs.RunGestureTimeouts();
        self.DispatchGeneratedGestures(&inputs)
    }
    fn DispatchGeneratedGestures(&mut self, inputs: &FrameAlignedInputQueue) -> io::Result<()> {
        for gesture in inputs.TakeGeneratedGestures() {
            let packet = interaction::frame_aligned_input_queue::OwnedInputPacket::FromCanonical(
                interaction::input_event::ConvertedWebInputEvent::Gesture(gesture.clone()),
            );
            self.DispatchInputWithoutLifecyclePacket(
                &InputEvent::Gesture(gesture),
                Some(packet.Borrow()),
            )?;
        }
        Ok(())
    }
    pub(super) fn PresentedFrameHasCurrentLayoutResult(&self) -> bool {
        self.frame.as_ref().is_some_and(|frame| {
            self.state
                .layout_engine
                .borrow()
                .GetLayoutResult()
                .is_some_and(|resident| {
                    resident.PtrEq(&frame.fragments)
                        && PaintFragmentCursor::TryFromHandle(&frame.fragments).is_some()
                })
        })
    }

    /// EventHandler requires a committed fragment/PrePaint snapshot before it
    /// enters hit testing. An ordinary input task is not itself a BeginFrame,
    /// but it cannot defer this synchronous lifecycle barrier to the next pulse:
    /// a retained LayoutResult handle may already have had its paint state
    /// invalidated by script, focus, or a geometry query.
    pub(super) fn CompleteLifecycleForImmediateInput(&mut self) -> io::Result<()> {
        let frame_current = self.PresentedFrameHasCurrentLayoutResult();
        if !self.state.dirty.get() && self.frame.is_some() && frame_current {
            return Ok(());
        }
        if self.frame.is_some() && !frame_current {
            // A shared resident paint sidecar can be superseded by a newer
            // Layout publication before Page has frozen its replacement frame.
            // Immediate EventHandler input requires PrePaint/Paint now; merely
            // entering UpdateFrameIfNeeded while `dirty == false` would return
            // the stale handle straight back to hit testing.
            self.state.dirty.set(true);
            self.state.scroll_only.set(false);
        }
        let executing = std::mem::replace(&mut self.begin_frame.executing, true);
        let completion_scope = self.open_engine.BeginRenderingOpportunity();
        let result = self.UpdateFrameIfNeeded();
        self.open_engine.EndRenderingOpportunity(completion_scope);
        self.begin_frame.executing = executing;
        self.RequestBeginFrameIfNeeded();
        result
    }

    fn AdvanceCompositorBeginFrame(&mut self, args: BeginFrameArgs) -> io::Result<()> {
        let output = {
            let mut rendering = self.state.rendering.borrow_mut();
            if !rendering.NeedsBeginFrames() {
                return Ok(());
            }
            rendering.BeginPhysicalFrame(args)?
        };
        self.ApplyCompositorWheelEffects(output.effects)?;
        if output.submitted {
            self.PresentCompositorOnlyFrame();
        }
        Ok(())
    }

    pub fn SetBeginFrameSource(&mut self, source: Option<Arc<dyn BeginFrameSource>>) {
        *self.state.begin_frame_source.borrow_mut() = source.clone();
        self.state.has_begin_frame_source.set(source.is_some());
        if let Some(scripts) = &self.scripts {
            scripts.SetBeginFrameSource(source.clone());
        }
        self.begin_frame.source = source;
        self.begin_frame.requested = false;
        self.begin_frame.last_frame = None;
        self.RequestBeginFrameIfNeeded();
    }
    pub fn QueueFrameInput(
        &mut self,
        input: InputEvent,
        queued_at: Instant,
        samples: usize,
    ) -> io::Result<()> {
        self.QueueFrameInputWithPacket(input, None, queued_at, samples, None)
    }
    pub fn QueueCanonicalFrameInput(
        &mut self,
        input: InputEvent,
        packet: interaction::frame_aligned_input_queue::OwnedInputPacket,
        queued_at: Instant,
        samples: usize,
    ) -> io::Result<()> {
        self.QueueFrameInputWithPacket(input, Some(packet), queued_at, samples, None)
    }
    pub fn QueueBrowserGesture(
        &mut self,
        gesture: interaction::input_event::WebGestureEvent,
        dispatch_id: u64,
        queued_at: Instant,
    ) -> io::Result<()> {
        let packet = interaction::frame_aligned_input_queue::OwnedInputPacket::FromCanonical(
            interaction::input_event::ConvertedWebInputEvent::Gesture(gesture.clone()),
        );
        self.QueueFrameInputWithPacket(
            InputEvent::Gesture(gesture),
            Some(packet),
            queued_at,
            1,
            Some(dispatch_id),
        )
    }
    pub fn TakeBrowserGestureDispositions(
        &self,
    ) -> Vec<interaction::frame_aligned_input_queue::BrowserGestureDisposition> {
        self.begin_frame.inputs.TakeBrowserGestureDispositions()
    }
    fn QueueFrameInputWithPacket(
        &mut self,
        input: InputEvent,
        packet: Option<interaction::frame_aligned_input_queue::OwnedInputPacket>,
        queued_at: Instant,
        samples: usize,
        browser_gesture_dispatch_id: Option<u64>,
    ) -> io::Result<()> {
        self.begin_frame
            .inputs
            .SetWidgetScale(self.state.constraints.borrow().layout_zoom_factor as f32);
        self.begin_frame.inputs.SetHasPointerRawUpdateEventHandlers(
            self.scripts
                .as_ref()
                .is_some_and(|scripts| scripts.HasPointerRawUpdateEventHandlers()),
        );
        // The impl CursorControlHandler is the sole browser exception that
        // forwards scroll gestures to MainThreadEventQueue. Its source owner
        // validates the cursor-control sequence before accepting the packet.
        let browser_cursor_control = browser_gesture_dispatch_id.is_some()
            && matches!(&input, InputEvent::Gesture(event) if event.IsGestureScroll());
        if !FrameAlignedInputQueue::IsQueueInput(&input) && !browser_cursor_control {
            self.FlushFrameInputs()?;
            let dispatch_started = Instant::now();
            let result = self.Dispatch(&input);
            let default_prevented = result.as_ref().is_ok_and(|result| result.default_prevented);
            self.begin_frame.dispatched.push(DispatchedFrameInput {
                input,
                queued_at,
                samples,
                default_prevented,
                dispatch_started,
                dispatch_finished: Instant::now(),
            });
            return result.map(|_| ());
        }
        if let Some(packet) = packet {
            if let Some(id) = browser_gesture_dispatch_id {
                self.begin_frame
                    .inputs
                    .PushBrowserGesturePacket(input, packet, queued_at, samples, id);
            } else {
                self.begin_frame
                    .inputs
                    .PushPacket(input, packet, queued_at, samples);
            }
        } else {
            self.begin_frame.inputs.Push(input, queued_at, samples);
        }
        if self.begin_frame.source.is_none() {
            self.FlushFrameInputs()
        } else if self.begin_frame.inputs.NeedsDispatchTask() {
            self.FlushFrameInputsWith(false)
        } else {
            self.RequestBeginFrameIfNeeded();
            Ok(())
        }
    }
    pub fn HasPendingFrameInput(&self) -> bool {
        !self.begin_frame.inputs.IsEmpty()
    }
    pub fn HasPendingWheelFrameInput(&self) -> bool {
        self.begin_frame.inputs.HasWheel()
    }
    pub fn TakeDispatchedFrameInputs(&mut self) -> Vec<DispatchedFrameInput> {
        std::mem::take(&mut self.begin_frame.dispatched)
    }
    pub fn TakeTouchDispositions(
        &self,
    ) -> Vec<interaction::frame_aligned_input_queue::TouchDisposition> {
        self.begin_frame.inputs.TakeTouchDispositions()
    }
    pub(super) fn RequestBeginFrameIfNeeded(&mut self) {
        if self.begin_frame.executing {
            return;
        }
        // Render-blocking styles/parser readiness is advanced by ordinary
        // loading tasks. Re-requesting an unrenderable dirty page each tick
        // cannot advance that work and must not compete with its task queue.
        // Once readiness changes, the next task reaches UpdateFrameIfNeeded.
        // Before the first PaintArtifact exists, cc has no active tree to
        // animate. Page readiness owns that first BeginMainFrame; admitting
        // cc's internal demand earlier can leave this owner-side request bit
        // attached to a no-update bootstrap transaction.
        let compositor_demand =
            self.frame.is_some() && self.state.rendering.borrow().NeedsBeginFrames();
        let demand = compositor_demand
            || (self.IsRenderingReady()
                && (self.begin_frame.inputs.NeedsBeginFrame()
                    || self.state.dirty.get()
                    || self.hover_state_dirty
                    || (self.scripts.is_some()
                        && !self.state.scroll_event_targets.borrow().is_empty())))
            || self.state.animation_begin_frame_requested.get()
            || self.resources.HasAnimatedImages()
            || self
                .scripts
                .as_ref()
                .is_some_and(|scripts| scripts.HasPendingAnimationFrames());
        if demand {
            if let Some(source) = &self.begin_frame.source {
                // The source owns request coalescing. A SetNeedsCommit request
                // can be absorbed by an already-pending compositor transaction
                // without delivering a BeginMainFrame to this Page; in that
                // case the local bit remains set, but later dirty work must be
                // allowed to restate its demand. Real sources are idempotent
                // while a physical pulse is pending.
                self.begin_frame.requested = true;
                source.request_begin_frame();
            }
        }
    }
    pub(super) fn DeferLifecycleForBeginFrame(&mut self) -> bool {
        if self.begin_frame.source.is_some() && !self.begin_frame.executing {
            self.RequestBeginFrameIfNeeded();
            true
        } else {
            false
        }
    }
    /// Run Page's rendering opportunity for a compositor-issued main frame.
    /// The platform only delivers the frame; Page owns input, animation,
    /// style/layout/paint ordering and the resulting immutable frame state.
    pub fn UpdateRendering(&mut self, args: BeginFrameArgs) -> io::Result<()> {
        // WebFrameWidgetImpl::BeginMainFrame keeps Blink's Oilpan allocation
        // scope live for the complete rendering opportunity.  Work before the
        // ordinary lifecycle update can still allocate: synthetic hover
        // preparation commits LayoutSelection and may lazily build an inline
        // OffsetMapping.  UpdateFrameIfNeeded has its own nested scope, but it
        // is too late for that pre-animation work.
        let mut heap_scope = foundation::LayoutHeapScope::new();
        heap_scope.DeferCollection();
        let mut trace = browser_tracing::span("lifecycle", "Page.UpdateRendering");
        trace.set("source_id", args.source_id as f64);
        trace.set("sequence", args.sequence_number as f64);
        trace.set("interval_ms", args.interval.as_secs_f64() * 1000.0);
        trace.set(
            "pending_wheel",
            self.HasPendingWheelFrameInput() as u8 as f64,
        );
        if self
            .begin_frame
            .last_frame
            .is_some_and(|(source, sequence)| {
                source == args.source_id && sequence >= args.sequence_number
            })
        {
            // The host may receive the same physical pulse through both its
            // direct and compositor routes. That pulse still acknowledges the
            // outstanding callback, but it cannot satisfy dirty work which was
            // requested after the original delivery. Clear Page's in-flight
            // bit and re-arm against the next frame instead of leaving it
            // permanently stuck at `requested = true`.
            self.begin_frame.requested = false;
            self.RequestBeginFrameIfNeeded();
            trace.set("duplicate", 1.0);
            return Ok(());
        }
        self.begin_frame.last_frame = Some((args.source_id, args.sequence_number));
        let render_start = self.state.InteractionEventClock().Now();
        let performance = self.state.window_performance_timing.EventTimingSource();
        performance.OnBeginMainFrame(args.source_id);
        performance.SetRenderStartTimeForPendingEvents(render_start);
        self.begin_frame.requested = false;
        self.state.animation_begin_frame_requested.set(false);
        let executing = std::mem::replace(&mut self.begin_frame.executing, true);
        // Image/font completions remain macro tasks. A rendering
        // opportunity must not add unrelated resource callbacks to input/rAF.
        let completion_scope = self.open_engine.BeginRenderingOpportunity();
        let result = (|| {
            let profile = std::env::var_os("BROWSER_PROFILE_INPUT")
                .is_some()
                .then(Instant::now);
            // A synchronous CSSOM geometry query may have advanced Layout
            // without running PrePaint/Paint.  That retires the fragment
            // result owned by the presented frame, so hit testing must first
            // finish the pending lifecycle and use the new canonical result.
            // This is only an ownership/currentness check; no fragment data is
            // copied across the Page boundary.
            let frame_layout_current = self.PresentedFrameHasCurrentLayoutResult();
            trace.set(
                "frame_layout_current_before_input",
                frame_layout_current as u8 as f64,
            );
            if self.frame.is_none() || !frame_layout_current {
                if self.frame.is_some() {
                    self.state.dirty.set(true);
                    self.state.scroll_only.set(false);
                }
                debug_assert!(
                    self.frame.is_none() || self.state.dirty.get(),
                    "a newer Layout result must keep the Page lifecycle dirty until Paint"
                );
                self.UpdateFrameAndStartVisibleDeferredImages()?;
            }
            if self.resources.HasAnimatedImages() {
                let mut client = ResourceClient {
                    state: self.state.clone(),
                    scripts: self.scripts.as_deref_mut(),
                };
                self.resources.SampleAnimatedImages(
                    args.frame_time,
                    args.sequence_number,
                    &mut client,
                )?;
            }
            // WebFrameWidgetImpl::BeginMainFrame performs this before Animate.
            // Clear first because listeners may force layout and mark it dirty
            // again for the following frame.
            if std::mem::take(&mut self.hover_state_dirty) && self.active {
                if self.cursor_position.is_some() {
                    // This synthetic move deliberately bypasses the ordinary
                    // input lifecycle wrapper. A compositor scroll can retain
                    // Layout geometry while replacing the fragment-local
                    // PrePaint revision, so prove resident layer order before
                    // entering EventHandler's no-lifecycle hit test.
                    let geometry = super::pointer_geometry::PagePointerGeometryProvider(
                        Rc::downgrade(&self.state),
                    );
                    interaction::adapter::page_pointer_hit_test::PagePointerGeometry::PrepareLifecycleExceptPaint(&geometry)
                        .map_err(|error| io::Error::other(format!(
                            "synthetic hover geometry: {error:?}"
                        )))?;
                    self.RecomputeMouseHoverStateWithoutLifecycle()?;
                }
            }
            let initial_done = profile.map(|start| start.elapsed());
            self.DrainFrameInputs()?;
            self.AdvanceCompositorBeginFrame(args)?;
            // Source: ProgrammaticScrollAnimator::TickAnimation. Smooth CSSOM
            // viewport scrolling samples on the rendering opportunity before
            // scroll events and requestAnimationFrame observe the new offset.
            self.state.TickProgrammaticScroll(args.frame_time);
            if let Some(runtime) = &self.interaction_scroll_runtime {
                runtime.Tick(args.frame_time);
            }
            self.state
                .local_frames
                .get()
                .unwrap()
                .TickInteractionScroll(args.frame_time);
            // AutoscrollController::Animate updates selection and invokes the
            // same resident LayoutBox autoscroll on each rendering opportunity.
            // Script listeners borrow this frame's runtime only for this call.
            let autoscroll_active = self
                .state
                .mouse_autoscroll
                .borrow()
                .upgrade()
                .is_some_and(|owner| owner.NeedsAnimation());
            if autoscroll_active {
                let compositor_now = self
                    .state
                    .rendering
                    .borrow()
                    .executor
                    .TimeTicks(args.frame_time);
                // Preserve the compositor clock origin while crossing into
                // Blink's independent source input timestamp value type.
                let autoscroll_now =
                    interaction::input::base::time::time::TimeTicks::FromInternalValue(
                        compositor_now.since_origin().InMicroseconds(),
                    );
                if let Some(scripts) = &mut self.scripts {
                    scripts.RunInteractionTask(
                        super::popup_tasks::PageInteractionTask::SelectionAutoscroll(
                            autoscroll_now,
                        ),
                    )?;
                } else {
                    self.interaction
                        .AnimateSelectionAutoscroll(&self.state.document.Handle(), autoscroll_now)
                        .map_err(io::Error::other)?;
                }
            }
            self.state
                .local_frames
                .get()
                .unwrap()
                .AnimateChildAutoscroll(args.frame_time)?;
            // The scroll event observes the new visual scroll offset. Commit
            // the retained scroll properties before script geometry queries;
            // otherwise getBoundingClientRect() would rebuild layout merely to
            // discover the offset that the compositor already applied.
            if !self.state.scroll_event_targets.borrow().is_empty() {
                self.state.ApplyPendingScrollUpdates(self.frame.as_mut());
            }
            // CSSOM View scroll steps run in the rendering opportunity before
            // requestAnimationFrame callbacks. Keep them off the ordinary task
            // queue so page loading cannot delay scroll-driven lazy loading.
            if let Some(scripts) = &mut self.scripts {
                scripts.DispatchPendingScrollEvents();
            }
            let input_done = profile.map(|start| start.elapsed());
            self.state
                .ApplyAnimationMutation(animation::AnimationMutation::BeginFrame {
                    source_id: args.source_id,
                    begin_frame_sequence: args.sequence_number,
                    frame_time: args.frame_time,
                });
            if let Some(scripts) = &mut self.scripts {
                scripts.RunAnimationFrameCallbacks(args.frame_time)?;
            }
            let animation_done = profile.map(|start| start.elapsed());
            self.UpdateFrameAndStartVisibleDeferredImages()?;
            let lifecycle_done = profile.map(|start| start.elapsed());
            self.UpdateCursor();
            if let Some(start) = profile {
                let total = start.elapsed();
                if total >= std::time::Duration::from_millis(16) {
                    let ms = |time: std::time::Duration| time.as_secs_f64() * 1000.0;
                    eprintln!(
                        "page-begin-frame-stages sequence={} initial_ms={:.3} input_ms={:.3} animation_ms={:.3} lifecycle_ms={:.3} cursor_ms={:.3} total_ms={:.3}",
                        args.sequence_number,
                        ms(initial_done.unwrap()),
                        ms(input_done.unwrap() - initial_done.unwrap()),
                        ms(animation_done.unwrap() - input_done.unwrap()),
                        ms(lifecycle_done.unwrap() - animation_done.unwrap()),
                        ms(total - lifecycle_done.unwrap()),
                        ms(total)
                    );
                }
            }
            Ok(())
        })();
        self.open_engine.EndRenderingOpportunity(completion_scope);
        self.begin_frame.executing = executing;
        self.RequestBeginFrameIfNeeded();
        trace.set("success", result.is_ok() as u8 as f64);
        trace.set(
            "presented_sequence",
            self.frame.as_ref().map_or(0, |frame| frame.sequence) as f64,
        );
        result
    }
    /// Receive real scroll input in an already-started native frame's late
    /// scroll window (cc's WAIT_FOR_SCROLL). Animation callbacks ran at the
    /// normal BeginFrame boundary and must not run again for this input.
    pub fn OnLateScrollInput(&mut self, args: BeginFrameArgs) -> io::Result<bool> {
        let mut trace = browser_tracing::span("lifecycle", "Page.OnLateScrollInput");
        trace.set("sequence", args.sequence_number as f64);
        if self.begin_frame.last_frame != Some((args.source_id, args.sequence_number))
            || self.begin_frame.executing
            || !self.begin_frame.inputs.HasWheel()
        {
            return Ok(false);
        }
        // Match kWaitForLateScrollEventsDeadlineRatio (0.333); this does not
        // extend the source's deadline or create a new rendering opportunity.
        let Some(window) = args
            .interval
            .checked_mul(333)
            .map(|interval| interval / 1000)
        else {
            return Ok(false);
        };
        let Some(deadline) = args.frame_time.checked_add(window) else {
            return Ok(false);
        };
        // The sole late-input admission check, immediately after queueing.
        // Once admitted, dispatch can cross this boundary without revocation.
        let dispatch_time = Instant::now();
        let deadline = deadline.min(args.deadline);
        let admitted = dispatch_time < deadline;
        trace.set(
            "dispatch_age_ms",
            dispatch_time
                .saturating_duration_since(args.frame_time)
                .as_secs_f64()
                * 1000.0,
        );
        trace.set(
            "deadline_age_ms",
            deadline
                .saturating_duration_since(args.frame_time)
                .as_secs_f64()
                * 1000.0,
        );
        trace.set("deadline_admitted", admitted as u8 as f64);
        if !admitted {
            return Ok(false);
        }
        let dispatched_before = self.begin_frame.dispatched.len();
        self.FlushFrameInputs()?;
        let handled = self.begin_frame.dispatched[dispatched_before..]
            .iter()
            .any(|entry| matches!(&entry.input, InputEvent::Wheel(_)));
        trace.set("wheel_handled", handled as u8 as f64);
        Ok(handled)
    }
    /// Preserve event order before an immediate event. This boundary performs
    /// lifecycle work, but animation callbacks still await their BeginFrame.
    pub fn FlushFrameInputs(&mut self) -> io::Result<()> {
        self.FlushFrameInputsWith(true)
    }

    fn FlushFrameInputsWith(&mut self, raf_aligned: bool) -> io::Result<()> {
        let _trace = browser_tracing::span("lifecycle", "Page.FlushFrameInputs");
        if !self.HasPendingFrameInput() {
            return Ok(());
        }
        let executing = std::mem::replace(&mut self.begin_frame.executing, true);
        let completion_scope = self.open_engine.BeginRenderingOpportunity();
        let result = (|| {
            let frame_layout_current = self.PresentedFrameHasCurrentLayoutResult();
            if self.frame.is_none() || !frame_layout_current {
                debug_assert!(
                    self.frame.is_none() || self.state.dirty.get(),
                    "a newer Layout result must keep the Page lifecycle dirty until Paint"
                );
                self.UpdateFrameAndStartVisibleDeferredImages()?;
            }
            self.DrainFrameInputsWith(raf_aligned)?;
            if !self.state.scroll_event_targets.borrow().is_empty() {
                self.state.ApplyPendingScrollUpdates(self.frame.as_mut());
            }
            if let Some(scripts) = &mut self.scripts {
                scripts.DispatchPendingScrollEvents();
            }
            self.UpdateFrameAndStartVisibleDeferredImages()?;
            self.UpdateCursor();
            Ok(())
        })();
        self.open_engine.EndRenderingOpportunity(completion_scope);
        self.begin_frame.executing = executing;
        self.RequestBeginFrameIfNeeded();
        result
    }
    fn DrainFrameInputs(&mut self) -> io::Result<()> {
        self.DrainFrameInputsWith(true)
    }

    fn DrainFrameInputsWith(&mut self, raf_aligned: bool) -> io::Result<()> {
        let mut trace = browser_tracing::span("lifecycle", "Page.DrainFrameInputs");
        let mut dispatched = 0usize;
        // Loading may not have produced any hit-test geometry yet. Keep these
        // events queued rather than dispatching against an invented empty page.
        if self.frame.is_none() {
            return Ok(());
        }
        // Clone only the queue owner so its client callback can synchronously
        // borrow Page. The translated queue retains the dispatch snapshot and
        // acknowledgement lifetime; no second host queue is materialized.
        let inputs = self.begin_frame.inputs.clone();
        let dispatch =
            |entry: interaction::frame_aligned_input_queue::QueuedFrameInput,
             packet: interaction::frame_aligned_input_queue::BorrowedInputPacket<'_>| {
                let dispatch_started = Instant::now();
                let _input_context = browser_tracing::enabled().then(|| {
                    browser_tracing::scope(browser_tracing::Context {
                        input_id: browser_tracing::instant_id(entry.queued_at),
                        ..Default::default()
                    })
                });
                let mut input_trace = browser_tracing::span("input", "Page.DispatchFrameInput");
                input_trace.set("samples", entry.samples as f64);
                input_trace.set(
                    "kind",
                    if matches!(&entry.input, InputEvent::Wheel(_)) {
                        1.0
                    } else {
                        2.0
                    },
                );
                if let InputEvent::Wheel(wheel) = &entry.input {
                    input_trace.set("delta_x", wheel.delta.x);
                    input_trace.set("delta_y", wheel.delta.y);
                    input_trace.set("compositor_owned", wheel.compositor_owned as u8 as f64);
                }
                if let InputEvent::Mouse(mouse) = &entry.input {
                    input_trace.set("mouse_type", mouse.r#type as u8 as f64);
                    input_trace.set("host_buttons", mouse.buttons as f64);
                    if let Some(web) = packet
                        .Event()
                        .AsAny()
                        .downcast_ref::<
                            interaction::input::public::common::input::web_mouse_event::WebMouseEvent,
                        >()
                    {
                        input_trace.set("web_button", web.pointer_properties.button as i16 as f64);
                        input_trace.set("web_modifiers", web.GetModifiers() as f64);
                        input_trace.set(
                            "web_x",
                            f64::from(web.pointer_properties.PositionInWidget().x()),
                        );
                        input_trace.set(
                            "web_y",
                            f64::from(web.pointer_properties.PositionInWidget().y()),
                        );
                    }
                }
                let result = self.DispatchInputWithoutLifecyclePacketFromBrowser(
                    &entry.input,
                    Some(packet),
                    entry.browser_gesture_dispatch_id.is_some(),
                );
                if result.is_ok() && matches!(&entry.input, InputEvent::Touch(_)) {
                    inputs.SetTouchAction(self.interaction.CurrentTouchActionForGesture());
                }
                let consumed = result.as_ref().is_ok_and(|result| {
                if matches!(&entry.input, InputEvent::Gesture(_)) {
                    result.input_result.map_or(result.default_prevented, |result| result != interaction::dispatch::public::platform::web_input_event_result::WebInputEventResult::kNotHandled)
                } else {
                    result.default_prevented
                }
            });
                let default_prevented =
                    result.as_ref().is_ok_and(|result| result.default_prevented);
                let result = result.map(|result| {
                    input_trace.set("default_prevented", result.default_prevented as u8 as f64);
                });
                self.begin_frame.dispatched.push(DispatchedFrameInput {
                    input: entry.input,
                    queued_at: entry.queued_at,
                    samples: entry.samples,
                    default_prevented,
                    dispatch_started,
                    dispatch_finished: Instant::now(),
                });
                dispatched += 1;
                trace.set("dispatched_inputs", dispatched as f64);
                // MainThreadEventQueue::DispatchRafAlignedInput dispatches the
                // queue snapshot without a lifecycle update between events.  A
                // wheel phase boundary prevents coalescing, but it does not make
                // style/layout/paint run between Ended and a following Began.
                // UpdateRendering resolves the complete batch below, after input
                // dispatch and before rAF observes the resulting state.
                result.map(|_| consumed)
            };
        inputs.TryDispatchPacket(raf_aligned, dispatch)?;
        self.DispatchGeneratedGestures(&inputs)
    }
}
