//! Owner-thread input queue corresponding to Blink's frame-aligned event queue.
//! Compatible updates coalesce across other continuous event classes; discrete
//! input and incompatible events of the same class remain ordering boundaries.
use crate::input_event::{InputEvent, MouseEventType, WheelPhase};
use std::{collections::VecDeque, time::Instant};

#[derive(Clone, Debug)]
pub struct QueuedFrameInput {
    pub input: InputEvent,
    pub queued_at: Instant,
    pub samples: usize,
}

#[derive(Clone, Debug)]
pub struct DispatchedFrameInput {
    pub input: InputEvent,
    pub queued_at: Instant,
    pub samples: usize,
    pub dispatch_started: Instant,
    pub dispatch_finished: Instant,
}

#[derive(Default)]
pub struct FrameAlignedInputQueue {
    entries: VecDeque<QueuedFrameInput>,
}
impl FrameAlignedInputQueue {
    pub fn IsFrameAligned(input: &InputEvent) -> bool {
        // Blink's MainThreadEventQueue classifies every WebMouseWheelEvent as
        // rAF-aligned, including phase boundaries.  Began/Ended still remain
        // ordering barriers below; they just wait for the same rendering
        // opportunity as the surrounding continuous samples instead of
        // forcing lifecycle work from the input callback.
        // Source: third_party/blink/renderer/platform/widget/input/
        //         main_thread_event_queue.cc:IsRafAlignedEvent.
        matches!(input, InputEvent::Wheel(_))
            || matches!(input, InputEvent::Mouse(event) if event.r#type == MouseEventType::kMove)
    }
    pub fn IsEmpty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn HasWheel(&self) -> bool {
        self.entries
            .iter()
            .any(|entry| matches!(&entry.input, InputEvent::Wheel(_)))
    }
    pub fn PopFront(&mut self) -> Option<QueuedFrameInput> {
        self.entries.pop_front()
    }
    pub fn Push(&mut self, input: InputEvent, queued_at: Instant, samples: usize) {
        // MainThreadEventQueue::QueuedWebInputEvent::FilterNewEvent keeps
        // searching across a different event class. Wheel/mouse alternation
        // must not turn one frame's continuous input into hundreds of tasks.
        for last in self.entries.iter_mut().rev() {
            if !Self::IsFrameAligned(&input) || !Self::IsFrameAligned(&last.input) {
                break;
            }
            let merged = match (&mut last.input, &input) {
                (InputEvent::Wheel(a), InputEvent::Wheel(b)) => {
                    let finite = |x: f64, y: f64| x.is_finite() && y.is_finite();
                    if matches!(a.phase, WheelPhase::kNone | WheelPhase::kChanged)
                        && a.CanCoalesce(b)
                        && a.position == b.position
                        && a.target_node_id == b.target_node_id
                        && finite(a.delta.x, b.delta.x)
                        && finite(a.delta.y, b.delta.y)
                        && (a.delta.x + b.delta.x).is_finite()
                        && (a.delta.y + b.delta.y).is_finite()
                    {
                        a.delta.x += b.delta.x;
                        a.delta.y += b.delta.y;
                        a.native = b.native.clone();
                        true
                    } else {
                        false
                    }
                }
                (InputEvent::Mouse(a), InputEvent::Mouse(b)) => {
                    if a.r#type == MouseEventType::kMove
                        && b.r#type == MouseEventType::kMove
                        && a.button == b.button
                        && a.modifiers == b.modifiers
                        && a.target_node_id == b.target_node_id
                    {
                        *a = b.clone();
                        true
                    } else {
                        false
                    }
                }
                _ => continue,
            };
            if merged {
                last.queued_at = last.queued_at.min(queued_at);
                last.samples = last.samples.saturating_add(samples);
                return;
            }
            // An incompatible event of the same class blocks coalescing
            // with still older events, as in FilterResult::StopIterating.
            break;
        }
        self.entries.push_back(QueuedFrameInput {
            input,
            queued_at,
            samples,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_event::{EventModifiers, WheelEvent, WheelPhase};
    use layoutng_assembly::internal::layout_input::Offset;
    #[test]
    fn begin_frame_wheel_coalescing_preserves_units_phase_and_modifiers() {
        let mut queue = FrameAlignedInputQueue::default();
        let time = Instant::now();
        let wheel = |dy, phase, shift| {
            InputEvent::Wheel(WheelEvent {
                delta: Offset { x: 0.0, y: dy },
                phase,
                modifiers: EventModifiers {
                    shift,
                    ..Default::default()
                },
                ..Default::default()
            })
        };
        for phase in [
            WheelPhase::kNone,
            WheelPhase::kBegan,
            WheelPhase::kChanged,
            WheelPhase::kEnded,
            WheelPhase::kCancelled,
        ] {
            assert!(FrameAlignedInputQueue::IsFrameAligned(&wheel(
                0.0, phase, false
            )));
        }
        queue.Push(wheel(10.0, WheelPhase::kChanged, false), time, 1);
        queue.Push(wheel(20.0, WheelPhase::kChanged, false), time, 2);
        queue.Push(wheel(-5.0, WheelPhase::kChanged, false), time, 1);
        queue.Push(wheel(-5.0, WheelPhase::kEnded, false), time, 1);
        queue.Push(wheel(-5.0, WheelPhase::kChanged, true), time, 1);
        let first = queue.PopFront().unwrap();
        assert_eq!(first.samples, 4);
        assert!(matches!(first.input, InputEvent::Wheel(event) if event.delta.y == 25.0));
        assert_eq!(queue.entries.len(), 2);
        let mut precise = wheel(-5.0, WheelPhase::kChanged, true);
        if let InputEvent::Wheel(event) = &mut precise {
            event.delta_units = crate::input_event::ScrollGranularity::kScrollByPrecisePixel;
        }
        queue.Push(precise, time, 1);
        assert_eq!(
            queue.entries.len(),
            3,
            "precise and non-precise input retain separate units"
        );
        let mut alternating = FrameAlignedInputQueue::default();
        for _ in 0..4 {
            alternating.Push(wheel(10., WheelPhase::kChanged, false), time, 1);
            alternating.Push(InputEvent::Mouse(Default::default()), time, 1);
        }
        assert_eq!(alternating.entries.len(), 2);
        let merged = alternating.PopFront().unwrap();
        assert_eq!(merged.samples, 4);
        assert!(matches!(merged.input, InputEvent::Wheel(event) if event.delta.y == 40.));
        assert_eq!(alternating.PopFront().unwrap().samples, 4);
    }
}
