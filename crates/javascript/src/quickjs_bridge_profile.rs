//! Opt-in aggregate attribution of the JS/native boundary. No per-call events.
use std::{cell::Cell, marker::PhantomData, rc::Rc, sync::OnceLock, time::Instant};

#[derive(Clone, Copy)]
pub(super) enum Phase {
    Call = 0,
    Arguments = 1,
    Return = 2,
    Graph = 3,
    GraphSource = 4,
    GraphApply = 5,
}
#[derive(Clone, Copy, Default)]
struct Totals {
    ns: [u64; 6],
    calls: [u64; 6],
    depth: [u32; 6],
    gc_mark: [u64; 3],
    gc_finalize: [u64; 3],
}
thread_local! { static TOTALS: Cell<Totals> = Cell::new(Totals::default()); }
fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os("BROWSER_PROFILE_HOST_BOUNDARY").is_some_and(|value| value == "1")
    }) && browser_tracing::enabled()
}
// Aggregate native tracing work on its owner thread. Values never escape GC;
// only scalar call/reference/capacity counts are retained for the entry report.
pub(super) fn native_gc_work(marking: bool, references: usize, capacity: usize) {
    if !enabled() {
        return;
    }
    TOTALS.with(|cell| {
        let mut totals = cell.get();
        let counters = if marking {
            &mut totals.gc_mark
        } else {
            &mut totals.gc_finalize
        };
        for (counter, delta) in counters
            .iter_mut()
            .zip([1, references as u64, capacity as u64])
        {
            *counter = counter.saturating_add(delta);
        }
        cell.set(totals);
    });
}
pub(super) struct Guard {
    phase: usize,
    started: Option<Instant>,
    _thread: PhantomData<Rc<()>>,
}
pub(super) fn enter(phase: Phase) -> Option<Guard> {
    if !enabled() {
        return None;
    }
    let phase = phase as usize;
    let outer = TOTALS.with(|cell| {
        let mut totals = cell.get();
        let outer = totals.depth[phase] == 0;
        totals.depth[phase] += 1;
        totals.calls[phase] = totals.calls[phase].saturating_add(1);
        cell.set(totals);
        outer
    });
    Some(Guard {
        phase,
        started: outer.then(Instant::now),
        _thread: PhantomData,
    })
}
impl Drop for Guard {
    fn drop(&mut self) {
        let elapsed = self
            .started
            .map(|start| start.elapsed().as_nanos().min(u64::MAX as u128) as u64);
        TOTALS.with(|cell| {
            let mut totals = cell.get();
            debug_assert!(totals.depth[self.phase] > 0);
            totals.depth[self.phase] -= 1;
            if let Some(elapsed) = elapsed {
                totals.ns[self.phase] = totals.ns[self.phase].saturating_add(elapsed);
            }
            cell.set(totals);
        });
    }
}
pub(super) struct Measurement(Totals);
impl Measurement {
    pub(super) fn start() -> Option<Self> {
        if !enabled() {
            return None;
        }
        let totals = TOTALS.with(Cell::get);
        // A native continuation can reenter JS. Its outer boundary has not
        // committed its duration yet; report that work on the owner entry,
        // rather than publishing zero/misleading deltas on the nested entry.
        (totals.depth[Phase::Call as usize] == 0).then_some(Self(totals))
    }
    pub(super) fn finish(start: Option<Self>, trace: &mut browser_tracing::Span) {
        let Some(start) = start else {
            return;
        };
        let end = TOTALS.with(Cell::get);
        for (phase, ns, calls) in [
            (0, "bridge_call_ns", "bridge_call_count"),
            (1, "bridge_args_ns", "bridge_args_count"),
            (2, "bridge_return_ns", "bridge_return_count"),
        ] {
            trace.set(ns, end.ns[phase].saturating_sub(start.0.ns[phase]) as f64);
            trace.set(
                calls,
                end.calls[phase].saturating_sub(start.0.calls[phase]) as f64,
            );
        }
        if end.gc_mark[0] != start.0.gc_mark[0] || end.gc_finalize[0] != start.0.gc_finalize[0] {
            browser_tracing::instant(
                "javascript",
                "HostGraph.GCWork",
                &[
                    (
                        "mark_calls",
                        end.gc_mark[0].saturating_sub(start.0.gc_mark[0]) as f64,
                    ),
                    (
                        "marked_references",
                        end.gc_mark[1].saturating_sub(start.0.gc_mark[1]) as f64,
                    ),
                    (
                        "marked_capacity",
                        end.gc_mark[2].saturating_sub(start.0.gc_mark[2]) as f64,
                    ),
                    (
                        "finalize_calls",
                        end.gc_finalize[0].saturating_sub(start.0.gc_finalize[0]) as f64,
                    ),
                    (
                        "finalized_references",
                        end.gc_finalize[1].saturating_sub(start.0.gc_finalize[1]) as f64,
                    ),
                    (
                        "finalized_capacity",
                        end.gc_finalize[2].saturating_sub(start.0.gc_finalize[2]) as f64,
                    ),
                ],
            );
        }
        browser_tracing::instant(
            "javascript",
            "HostGraph.BoundaryTotals",
            &[
                ("sync_ns", end.ns[3].saturating_sub(start.0.ns[3]) as f64),
                (
                    "sync_count",
                    end.calls[3].saturating_sub(start.0.calls[3]) as f64,
                ),
                ("source_ns", end.ns[4].saturating_sub(start.0.ns[4]) as f64),
                (
                    "source_count",
                    end.calls[4].saturating_sub(start.0.calls[4]) as f64,
                ),
                ("apply_ns", end.ns[5].saturating_sub(start.0.ns[5]) as f64),
                (
                    "apply_count",
                    end.calls[5].saturating_sub(start.0.calls[5]) as f64,
                ),
            ],
        );
    }
}
