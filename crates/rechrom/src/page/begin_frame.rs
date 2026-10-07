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
impl Page {
    pub fn SetBeginFrameSource(&mut self, source: Option<Arc<dyn BeginFrameSource>>) {
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
        let aligned = FrameAlignedInputQueue::IsFrameAligned(&input);
        if !aligned {
            self.FlushFrameInputs()?;
        }
        self.begin_frame.inputs.Push(input, queued_at, samples);
        if self.begin_frame.source.is_none() || !aligned {
            self.FlushFrameInputs()
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
    pub(super) fn RequestBeginFrameIfNeeded(&mut self) {
        if self.begin_frame.requested || self.begin_frame.executing {
            return;
        }
        // Render-blocking styles/parser readiness is advanced by ordinary
        // loading tasks. Re-requesting an unrenderable dirty page each tick
        // cannot advance that work and must not compete with its task queue.
        // Once readiness changes, RunTasks reaches UpdateFrameIfNeeded again.
        let demand = (self.IsRenderingReady()
            && (self.HasPendingFrameInput() || self.state.dirty.get() || self.hover_state_dirty))
            || self.resources.HasAnimatedImages()
            || self
                .scripts
                .as_ref()
                .is_some_and(|scripts| scripts.HasPendingAnimationFrames());
        if demand {
            if let Some(source) = &self.begin_frame.source {
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
    pub fn OnBeginFrame(&mut self, args: BeginFrameArgs) -> io::Result<()> {
        let mut trace = browser_tracing::span("lifecycle", "Page.OnBeginFrame");
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
            trace.set("duplicate", 1.0);
            return Ok(());
        }
        self.begin_frame.last_frame = Some((args.source_id, args.sequence_number));
        self.begin_frame.requested = false;
        let executing = std::mem::replace(&mut self.begin_frame.executing, true);
        // Image/font completions remain macro tasks in RunTasks. A rendering
        // opportunity must not add unrelated resource callbacks to input/rAF.
        let budget = self.resource_completion_budget.replace(0);
        let result = (|| {
            let profile = std::env::var_os("BROWSER_PROFILE_INPUT")
                .is_some()
                .then(Instant::now);
            if self.frame.is_none() {
                self.UpdateFrameIfNeeded()?;
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
                if let Some(position) = self.cursor_position {
                    self.DispatchInputWithoutLifecycle(&InputEvent::Mouse(
                        interaction::input_event::MouseEvent {
                            r#type: interaction::input_event::MouseEventType::kMove,
                            position,
                            ..Default::default()
                        },
                    ))?;
                }
            }
            let initial_done = profile.map(|start| start.elapsed());
            self.DrainFrameInputs()?;
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
            if let Some(scripts) = &mut self.scripts {
                scripts.RunAnimationFrameCallbacks(args.frame_time)?;
            }
            let animation_done = profile.map(|start| start.elapsed());
            self.UpdateFrameIfNeeded()?;
            let lifecycle_done = profile.map(|start| start.elapsed());
            self.UpdateCursor();
            if let Some(start) = profile {
                let total = start.elapsed();
                if total >= std::time::Duration::from_millis(16) {
                    let ms = |time: std::time::Duration| time.as_secs_f64() * 1000.0;
                    eprintln!("page-begin-frame-stages sequence={} initial_ms={:.3} input_ms={:.3} animation_ms={:.3} lifecycle_ms={:.3} cursor_ms={:.3} total_ms={:.3}",
                        args.sequence_number, ms(initial_done.unwrap()),
                        ms(input_done.unwrap() - initial_done.unwrap()),
                        ms(animation_done.unwrap() - input_done.unwrap()),
                        ms(lifecycle_done.unwrap() - animation_done.unwrap()),
                        ms(total - lifecycle_done.unwrap()), ms(total));
                }
            }
            Ok(())
        })();
        self.resource_completion_budget = budget;
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
        let _trace = browser_tracing::span("lifecycle", "Page.FlushFrameInputs");
        if !self.HasPendingFrameInput() {
            return Ok(());
        }
        let executing = std::mem::replace(&mut self.begin_frame.executing, true);
        let budget = self.resource_completion_budget.replace(0);
        let result = (|| {
            if self.frame.is_none() {
                self.UpdateFrameIfNeeded()?;
            }
            self.DrainFrameInputs()?;
            if !self.state.scroll_event_targets.borrow().is_empty() {
                self.state.ApplyPendingScrollUpdates(self.frame.as_mut());
            }
            if let Some(scripts) = &mut self.scripts {
                scripts.DispatchPendingScrollEvents();
            }
            self.UpdateFrameIfNeeded()?;
            self.UpdateCursor();
            Ok(())
        })();
        self.resource_completion_budget = budget;
        self.begin_frame.executing = executing;
        self.RequestBeginFrameIfNeeded();
        result
    }
    fn DrainFrameInputs(&mut self) -> io::Result<()> {
        let mut trace = browser_tracing::span("lifecycle", "Page.DrainFrameInputs");
        let mut dispatched = 0usize;
        // Loading may not have produced any hit-test geometry yet. Keep these
        // events queued rather than dispatching against an invented empty page.
        if self.frame.is_none() {
            return Ok(());
        }
        while let Some(entry) = self.begin_frame.inputs.PopFront() {
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
            }
            let result = self.DispatchInputWithoutLifecycle(&entry.input);
            let result = result.and_then(|result| {
                input_trace.set("default_prevented", result.default_prevented as u8 as f64);
                if !result.default_prevented {
                    if let InputEvent::Wheel(wheel) = &entry.input {
                        self.ApplyWheelDefaultWithoutLifecycle(wheel)?;
                    }
                }
                Ok(())
            });
            self.begin_frame.dispatched.push(DispatchedFrameInput {
                input: entry.input,
                queued_at: entry.queued_at,
                samples: entry.samples,
                dispatch_started,
                dispatch_finished: Instant::now(),
            });
            dispatched += 1;
            trace.set("dispatched_inputs", dispatched as f64);
            result?;
            // Entries left after coalescing have an event boundary. Resolve
            // prior mutations before the next hit test/default scroll route.
            if self.HasPendingFrameInput() {
                self.UpdateFrameIfNeeded()?;
            }
        }
        Ok(())
    }
}
