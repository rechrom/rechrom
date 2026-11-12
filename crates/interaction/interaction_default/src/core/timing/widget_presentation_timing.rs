//! Chromium WebFrameWidgetImpl.cc:4001-4098 (ReportTimeSwapPromise callbacks),
//! LayerTreeView.cc:413-434,518-522 and layer_tree_view.h:186-192.
use super::window_performance::{
    PresentationTimeRequest, PresentationTimingDetails, WindowPerformance,
};
use interaction_input::base::time::time::TimeTicks;
use std::{
    collections::{HashMap, VecDeque},
    rc::{Rc, Weak},
};
enum Registration {
    Event {
        performance: Weak<WindowPerformance>,
        request: PresentationTimeRequest,
    },
    Paint {
        client: Weak<dyn super::paint_timing_presentation::PaintTimingPresentationClient>,
        request: u64,
    },
}
impl Registration {
    fn Resolve(self, details: PresentationTimingDetails) {
        match self {
            Self::Event {
                performance,
                request,
            } => {
                if let Some(performance) = performance.upgrade() {
                    performance.OnPresentationPromiseResolved(request, details);
                }
            }
            Self::Paint { client, request } => {
                if let Some(client) = client.upgrade() {
                    client.OnPaintPresentation(request, details);
                }
            }
        }
    }
}
/// Callback ownership is moved out before Page releases its widget borrow.
pub struct WidgetPresentationCallback {
    registration: Registration,
    details: PresentationTimingDetails,
}
impl WidgetPresentationCallback {
    pub fn Run(self) {
        self.registration.Resolve(self.details);
    }
}
struct AfterSwap {
    registration: Registration,
    swap_time: TimeTicks,
}
/// This is the FrameWidget/LayerTreeView owner, and survives Document swaps.
/// Registrations contain weak original WindowPerformance identities.
pub struct WidgetPresentationTiming {
    next_request: u64,
    unswapped: HashMap<u64, Registration>,
    callbacks: VecDeque<(u32, Vec<AfterSwap>)>,
}
impl WidgetPresentationTiming {
    pub fn new() -> Self {
        Self {
            next_request: 0,
            unswapped: HashMap::new(),
            callbacks: VecDeque::new(),
        }
    }
    pub fn Register(
        &mut self,
        performance: &Rc<WindowPerformance>,
        request: PresentationTimeRequest,
    ) -> u64 {
        self.next_request = self
            .next_request
            .checked_add(1)
            .expect("widget presentation request identity");
        let id = self.next_request;
        self.unswapped.insert(
            id,
            Registration::Event {
                performance: Rc::downgrade(performance),
                request,
            },
        );
        id
    }
    pub fn RegisterPaint(
        &mut self,
        client: Weak<dyn super::paint_timing_presentation::PaintTimingPresentationClient>,
        request: u64,
    ) -> u64 {
        self.next_request = self
            .next_request
            .checked_add(1)
            .expect("widget presentation request identity");
        let id = self.next_request;
        self.unswapped
            .insert(id, Registration::Paint { client, request });
        id
    }
    pub fn DidSwap(&mut self, request: u64, frame_token: u32, swap_time: TimeTicks) {
        let Some(registration) = self.unswapped.remove(&request) else {
            return;
        };
        assert!(frame_token > 0);
        assert!(!swap_time.is_null());
        if self
            .callbacks
            .back()
            .is_none_or(|(token, _)| *token != frame_token)
        {
            if let Some((token, _)) = self.callbacks.back() {
                assert!(*token < frame_token);
            }
            self.callbacks.push_back((frame_token, Vec::new()));
        }
        self.callbacks.back_mut().unwrap().1.push(AfterSwap {
            registration,
            swap_time,
        });
    }
    pub fn DidNotSwap(&mut self, request: u64, time: TimeTicks) {
        if let Some(callback) = self.TakeDidNotSwapCallback(request, time) {
            callback.Run();
        }
    }
    pub fn TakeDidNotSwapCallback(
        &mut self,
        request: u64,
        time: TimeTicks,
    ) -> Option<WidgetPresentationCallback> {
        let registration = self.unswapped.remove(&request)?;
        Some(WidgetPresentationCallback {
            registration,
            details: PresentationTimingDetails {
                timestamp: time,
                actual_frame_source_id: 0,
                manual_begin_frame: false,
                actual_source_is_manual: false,
                ignore_unexpected_frame_source: false,
            },
        })
    }
    /// Only a real successful backend presentation reaches this callback. A
    /// swap acknowledgement or Unavailable cannot consume the registration.
    pub fn DidPresentCompositorFrame(
        &mut self,
        frame_token: u32,
        details: PresentationTimingDetails,
    ) {
        for callback in self.TakePresentedCallbacks(frame_token, details) {
            callback.Run();
        }
    }
    pub fn TakePresentedCallbacks(
        &mut self,
        frame_token: u32,
        details: PresentationTimingDetails,
    ) -> Vec<WidgetPresentationCallback> {
        let mut ready = Vec::new();
        while self
            .callbacks
            .front()
            .is_some_and(|(token, _)| *token <= frame_token)
        {
            let (_, callbacks) = self.callbacks.pop_front().unwrap();
            for callback in callbacks {
                let mut details = details;
                if details.timestamp.is_null() || details.timestamp <= callback.swap_time {
                    details.timestamp = callback.swap_time;
                }
                ready.push(WidgetPresentationCallback {
                    registration: callback.registration,
                    details,
                });
            }
        }
        ready
    }
    pub fn PendingCallbackCount(&self) -> usize {
        self.unswapped.len() + self.callbacks.iter().map(|(_, c)| c.len()).sum::<usize>()
    }
}
