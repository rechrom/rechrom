//! Owner-thread input queue corresponding to Blink's frame-aligned event queue.
//! Only adjacent compatible updates coalesce; gesture boundaries remain events.
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
        matches!(input, InputEvent::Wheel(event)
            if matches!(event.phase, WheelPhase::kNone | WheelPhase::kChanged))
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
        if let Some(last) = self.entries.back_mut() {
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
                _ => false,
            };
            if merged {
                last.queued_at = last.queued_at.min(queued_at);
                last.samples = last.samples.saturating_add(samples);
                return;
            }
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
    use crate::input_event::{EventModifiers, WheelEvent};
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
    }
}
