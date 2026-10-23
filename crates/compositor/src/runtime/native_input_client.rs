//! Native compositor event queue bound to cc's InputHandlerClient boundary.
use crate::{
    adapter::{
        begin_frame::BeginFrameArgs,
        geometry::{PointF, SizeF},
        time::TimeTicks,
    },
    cc::input::{
        input_handler::{InputHandlerClient, ScrollEventDispatchMode},
        scroll_state_data::ScrollStateData,
    },
};
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ScrollUpdateRoute {
    Dispatch,
    Enqueue,
    EnqueueAndRequestAnimation,
}

struct QueueState {
    queued: Vec<ScrollStateData>,
    mode: ScrollEventDispatchMode,
    deadline_ratio: f64,
    current_begin_frame: BeginFrameArgs,
    enqueue_scroll_events: bool,
    has_seen_first_scroll_update_after_begin: bool,
}

impl Default for QueueState {
    fn default() -> Self {
        Self {
            queued: Vec::new(),
            mode: ScrollEventDispatchMode::kEnqueueScrollEvents,
            deadline_ratio: 0.333,
            current_begin_frame: BeginFrameArgs::default(),
            enqueue_scroll_events: true,
            has_seen_first_scroll_update_after_begin: true,
        }
    }
}

#[derive(Default)]
pub(crate) struct NativeInputHandlerClient {
    state: Mutex<QueueState>,
}

impl NativeInputHandlerClient {
    pub(crate) fn BeginGesture(&self) {
        self.state
            .lock()
            .unwrap()
            .has_seen_first_scroll_update_after_begin = false;
    }

    pub(crate) fn RouteUpdate(&self, input: ScrollStateData, now: TimeTicks) -> ScrollUpdateRoute {
        let mut state = self.state.lock().unwrap();
        // Source: blink InputHandlerProxy::RouteToTypeSpecificHandler. The
        // first touchpad GestureScrollUpdate is dispatched immediately even
        // when ordinary updates are frame-aligned. This establishes the
        // compositor-owned gesture without adding a one-frame startup delay.
        if !state.has_seen_first_scroll_update_after_begin {
            state.has_seen_first_scroll_update_after_begin = true;
            if state.queued.is_empty() {
                state.enqueue_scroll_events = true;
                return ScrollUpdateRoute::Dispatch;
            }
        }
        let dispatch_before_deadline = matches!(
            state.mode,
            ScrollEventDispatchMode::kDispatchScrollEventsImmediately
                | ScrollEventDispatchMode::kUseScrollPredictorForDeadline
                | ScrollEventDispatchMode::kDispatchScrollEventsUntilDeadline
        );
        if dispatch_before_deadline && !state.enqueue_scroll_events {
            // Source: blink InputHandlerProxy::RouteToTypeSpecificHandler and
            // ShouldNotDispatchLateInputEvent. Once one late event is selected,
            // all subsequent events return to the queue for the next frame.
            state.enqueue_scroll_events = true;
            let args = &state.current_begin_frame;
            let frame_age = now - args.frame_time;
            let deadline = args.interval * state.deadline_ratio;
            let after_deadline = matches!(
                state.mode,
                ScrollEventDispatchMode::kUseScrollPredictorForDeadline
                    | ScrollEventDispatchMode::kDispatchScrollEventsUntilDeadline
            ) && args.IsValid()
                && frame_age <= args.interval
                && frame_age > deadline;
            if !after_deadline {
                return ScrollUpdateRoute::Dispatch;
            }
        }
        let queue_was_empty = state.queued.is_empty();
        state.queued.push(input);
        let route = if queue_was_empty {
            ScrollUpdateRoute::EnqueueAndRequestAnimation
        } else {
            ScrollUpdateRoute::Enqueue
        };
        route
    }

    pub(crate) fn TakeQueued(&self) -> Vec<ScrollStateData> {
        std::mem::take(&mut self.state.lock().unwrap().queued)
    }
}

impl InputHandlerClient for NativeInputHandlerClient {
    fn WillShutdown(&self) {}
    fn Animate(&self, _time: TimeTicks) {}
    fn ReconcileElasticOverscrollAndRootScroll(&self) {
        // Source: blink InputHandlerProxy::ReconcileElasticOverscrollAndRootScroll.
        // Starting draw closes the one-event late-input window for this frame;
        // subsequent scroll updates belong to the compositor queue.
        self.state.lock().unwrap().enqueue_scroll_events = true;
    }
    fn SetPrefersReducedMotion(&self, _reduced: bool) {}
    fn UpdateRootLayerStateForSynchronousInputHandler(
        &self,
        _offset: PointF,
        _max: PointF,
        _size: SizeF,
        _scale: f32,
        _min: f32,
        _max_scale: f32,
    ) {
    }
    fn DeliverInputForBeginFrame(&self, args: &BeginFrameArgs) -> Vec<ScrollStateData> {
        let mut state = self.state.lock().unwrap();
        state.current_begin_frame = args.clone();
        state.enqueue_scroll_events = !state.queued.is_empty();
        let queued = std::mem::take(&mut state.queued);
        queued
    }
    fn DeliverInputForHighLatencyMode(&self) {}
    fn DeliverInputForDeadline(&self) {}
    fn DidFinishImplFrame(&self) {
        let mut state = self.state.lock().unwrap();
        state.enqueue_scroll_events = true;
    }
    fn HasQueuedInput(&self) -> bool {
        !self.state.lock().unwrap().queued.is_empty()
    }
    fn SetScrollEventDispatchMode(&self, mode: ScrollEventDispatchMode, ratio: f64) {
        let mut state = self.state.lock().unwrap();
        state.mode = mode;
        state.deadline_ratio = ratio;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starting_draw_closes_the_late_input_dispatch_window() {
        let client = NativeInputHandlerClient::default();
        client.SetScrollEventDispatchMode(
            ScrollEventDispatchMode::kDispatchScrollEventsUntilDeadline,
            0.333,
        );

        // An empty frame opens Chromium's one-event late-input window.
        assert!(client
            .DeliverInputForBeginFrame(&BeginFrameArgs::default())
            .is_empty());

        // Beginning draw closes it even when no scroll event arrived first.
        // The next update must wait for the following BeginFrame.
        client.ReconcileElasticOverscrollAndRootScroll();
        assert_eq!(
            client.RouteUpdate(ScrollStateData::default(), TimeTicks::default()),
            ScrollUpdateRoute::EnqueueAndRequestAnimation
        );
        assert_eq!(
            client.RouteUpdate(ScrollStateData::default(), TimeTicks::default()),
            ScrollUpdateRoute::Enqueue
        );
        assert_eq!(client.TakeQueued().len(), 2);
    }

    #[test]
    fn first_touchpad_update_after_begin_dispatches_immediately() {
        let client = NativeInputHandlerClient::default();
        client.SetScrollEventDispatchMode(
            ScrollEventDispatchMode::kDispatchScrollEventsUntilDeadline,
            0.333,
        );
        client.BeginGesture();
        assert_eq!(
            client.RouteUpdate(ScrollStateData::default(), TimeTicks::default()),
            ScrollUpdateRoute::Dispatch
        );
    }
}
