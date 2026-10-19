#![allow(non_snake_case)]
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameTiming {
    pub frame_time: Instant,
    pub interval: Duration,
    pub source_deadline: Instant,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameDeadlines {
    pub compositor_prepare: Instant,
    pub draw_and_swap: Instant,
}

impl FrameTiming {
    pub fn Deadlines(self) -> FrameDeadlines {
        FrameDeadlines {
            compositor_prepare: self.frame_time + self.interval / 3,
            draw_and_swap: self
                .source_deadline
                .checked_sub(self.interval / 3)
                .unwrap_or(self.frame_time)
                .max(self.frame_time),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ScheduledFrame<T> {
    payload: T,
    deadlines: FrameDeadlines,
    prepared: bool,
    generation: u64,
    candidate_produced: bool,
    retry_pending: bool,
    retry_consumed: bool,
}

impl<T: Copy> ScheduledFrame<T> {
    pub fn Payload(self) -> T {
        self.payload
    }

    pub fn IsPrepared(self) -> bool {
        self.prepared
    }
}

/// Pure Viz deadline state. The embedding runtime supplies time and executes
/// the returned work; this type owns policy without owning a clock or thread.
pub struct DisplayScheduler<T> {
    pending: Option<ScheduledFrame<T>>,
    generation: u64,
}

impl<T> Default for DisplayScheduler<T> {
    fn default() -> Self {
        Self {
            pending: None,
            generation: 0,
        }
    }
}

impl<T: Copy> DisplayScheduler<T> {
    pub fn BeginFrame(&mut self, payload: T, timing: FrameTiming) -> Option<ScheduledFrame<T>> {
        self.generation = self.generation.wrapping_add(1);
        self.pending.replace(ScheduledFrame {
            payload,
            deadlines: timing.Deadlines(),
            prepared: false,
            generation: self.generation,
            candidate_produced: false,
            retry_pending: false,
            retry_consumed: false,
        })
    }

    pub fn NeedsPrepare(&self, now: Instant) -> bool {
        self.pending.is_some_and(|frame| {
            (!frame.prepared && now >= frame.deadlines.compositor_prepare)
                || (frame.retry_pending && now < frame.deadlines.draw_and_swap)
        })
    }

    /// Genuine active drawable damage can reopen one clean early preparation.
    /// Generic future-frame demand (including swap completion) cannot do so.
    pub fn NotifyVisualDamage(&mut self, now: Instant) {
        if let Some(frame) = &mut self.pending {
            if frame.prepared
                && !frame.candidate_produced
                && !frame.retry_consumed
                && now < frame.deadlines.draw_and_swap
            {
                frame.retry_pending = true;
            }
        }
    }

    pub fn PendingFrame(&self) -> Option<ScheduledFrame<T>> {
        self.pending
    }

    /// Results from a retired cycle must never latch a replacement cycle.
    pub fn MarkPrepared(&mut self, attempted: ScheduledFrame<T>, produced: bool) {
        if let Some(frame) = &mut self.pending {
            if frame.generation != attempted.generation {
                return;
            }
            frame.retry_consumed |= frame.prepared;
            frame.prepared = true;
            frame.retry_pending = false;
            frame.candidate_produced |= produced;
        }
    }

    pub fn PendingPayload(&self) -> Option<T> {
        self.pending.map(|frame| frame.payload)
    }

    pub fn TakeIfDrawDue(&mut self, now: Instant) -> Option<T> {
        self.pending
            .filter(|frame| now >= frame.deadlines.draw_and_swap)
            .and_then(|_| self.pending.take())
            .map(|frame| frame.payload)
    }

    pub fn NextDeadline(&self) -> Option<Instant> {
        self.pending.map(|frame| {
            if frame.prepared && !frame.retry_pending {
                frame.deadlines.draw_and_swap
            } else {
                frame.deadlines.compositor_prepare
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scheduler_owns_prepare_and_draw_deadline_policy() {
        let start = Instant::now();
        let interval = Duration::from_millis(15);
        let mut scheduler = DisplayScheduler::default();
        assert!(scheduler
            .BeginFrame(
                7u64,
                FrameTiming {
                    frame_time: start,
                    interval,
                    source_deadline: start + interval,
                },
            )
            .is_none());
        assert_eq!(
            scheduler.NextDeadline(),
            Some(start + Duration::from_millis(5))
        );
        assert!(!scheduler.NeedsPrepare(start + Duration::from_millis(4)));
        assert!(scheduler.NeedsPrepare(start + Duration::from_millis(5)));
        scheduler.MarkPrepared(scheduler.PendingFrame().unwrap(), true);
        assert_eq!(
            scheduler.NextDeadline(),
            Some(start + Duration::from_millis(10))
        );
        assert_eq!(
            scheduler.TakeIfDrawDue(start + Duration::from_millis(9)),
            None
        );
        assert_eq!(
            scheduler.TakeIfDrawDue(start + Duration::from_millis(10)),
            Some(7)
        );
        // Characterize before/middle/at-deadline damage without changing the
        // deadline constants. A clean attempt can reopen only once.
        let timing = FrameTiming {
            frame_time: start,
            interval,
            source_deadline: start + interval,
        };
        for (offset, expected_retry) in [(4, false), (7, true), (10, false), (11, false)] {
            scheduler.BeginFrame(8, timing);
            if offset >= 5 {
                scheduler.MarkPrepared(scheduler.PendingFrame().unwrap(), false);
            }
            let now = start + Duration::from_millis(offset);
            scheduler.NotifyVisualDamage(now);
            scheduler.NotifyVisualDamage(now);
            assert_eq!(scheduler.NeedsPrepare(now), expected_retry);
            if expected_retry {
                scheduler.MarkPrepared(scheduler.PendingFrame().unwrap(), false);
                scheduler.NotifyVisualDamage(now);
                assert!(
                    !scheduler.NeedsPrepare(now),
                    "one retry even when still clean"
                );
            }
        }
        scheduler.BeginFrame(9, timing);
        let stale = scheduler.PendingFrame().unwrap();
        scheduler.MarkPrepared(stale, true);
        scheduler.MarkPrepared(stale, false); // production is latched
        scheduler.NotifyVisualDamage(start + Duration::from_millis(7));
        assert!(!scheduler.NeedsPrepare(start + Duration::from_millis(7)));
        scheduler.BeginFrame(10, timing);
        scheduler.MarkPrepared(stale, true); // retired cycle cannot mark new one
        assert!(scheduler.NeedsPrepare(start + Duration::from_millis(5)));
        scheduler.MarkPrepared(scheduler.PendingFrame().unwrap(), false);
        scheduler.NotifyVisualDamage(start + Duration::from_millis(7));
        assert!(
            !scheduler.NeedsPrepare(start + Duration::from_millis(10)),
            "late execution does not retry"
        );
        assert_eq!(
            scheduler.TakeIfDrawDue(start + Duration::from_millis(10)),
            Some(10)
        );
    }
}
