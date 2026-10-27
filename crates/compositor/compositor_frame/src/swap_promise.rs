//! Frame submission promise boundary.

use compositor_scheduler::TimeTicks;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DidNotSwapReason {
    SWAP_FAILS,
    COMMIT_FAILS,
    COMMIT_NO_UPDATE,
    ACTIVATION_FAILS,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DidNotSwapAction {
    BREAK_PROMISE,
    KEEP_ACTIVE,
}

pub trait SwapPromise<M: ?Sized> {
    fn DidActivate(&mut self);
    fn WillSwap(&mut self, metadata: &mut M);
    fn DidSwap(&mut self);
    fn DidNotSwap(&mut self, reason: DidNotSwapReason, timestamp: TimeTicks) -> DidNotSwapAction;
    fn OnCommit(&mut self) {}
    fn GetTraceId(&self) -> i64;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SwapPromiseStats {
    pub visited: usize,
    pub kept_active: usize,
    pub completed: usize,
    pub broken: usize,
}

pub fn PassSwapPromises<P, M: ?Sized>(
    current: &mut Vec<Option<Box<P>>>,
    mut incoming: Vec<Box<P>>,
    timestamp: TimeTicks,
) -> SwapPromiseStats
where
    P: SwapPromise<M>,
{
    let mut trace = browser_tracing::span("compositor.frame", "Compositor.Frame.PassSwapPromises");
    let mut stats = SwapPromiseStats::default();
    for promise in current.iter_mut() {
        stats.visited += 1;
        if promise
            .as_mut()
            .expect("non-null swap promise")
            .DidNotSwap(DidNotSwapReason::SWAP_FAILS, timestamp)
            == DidNotSwapAction::KEEP_ACTIVE
        {
            stats.kept_active += 1;
            incoming.push(promise.take().unwrap());
        } else {
            stats.broken += 1;
        }
    }
    current.clear();
    *current = incoming.into_iter().map(Some).collect();
    trace.set("visited", stats.visited as f64);
    trace.set("kept_active", stats.kept_active as f64);
    trace.set("broken", stats.broken as f64);
    stats
}

pub fn FinishSwapPromises<P, M: ?Sized>(
    regular: &mut [Option<Box<P>>],
    pinned: &mut [Option<Box<P>>],
    metadata: &mut M,
) -> SwapPromiseStats
where
    P: SwapPromise<M>,
{
    let mut trace =
        browser_tracing::span("compositor.frame", "Compositor.Frame.FinishSwapPromises");
    let mut stats = SwapPromiseStats::default();
    for promise in regular.iter_mut().chain(pinned.iter_mut()) {
        stats.visited += 1;
        promise
            .as_mut()
            .expect("non-null swap promise")
            .WillSwap(metadata);
    }
    trace.set("visited", stats.visited as f64);
    stats
}

pub fn ClearSwapPromises<P, M: ?Sized>(
    regular: &mut Vec<Option<Box<P>>>,
    pinned: &mut Vec<Option<Box<P>>>,
) -> SwapPromiseStats
where
    P: SwapPromise<M>,
{
    let mut trace = browser_tracing::span("compositor.frame", "Compositor.Frame.ClearSwapPromises");
    let mut stats = SwapPromiseStats::default();
    for promise in regular.iter_mut().chain(pinned.iter_mut()) {
        stats.visited += 1;
        stats.completed += 1;
        promise.as_mut().expect("non-null swap promise").DidSwap();
    }
    regular.clear();
    pinned.clear();
    trace.set("completed", stats.completed as f64);
    stats
}

fn BreakSwapPromiseList<P, M: ?Sized>(
    promises: &mut Vec<Option<Box<P>>>,
    reason: DidNotSwapReason,
    timestamp: TimeTicks,
) -> SwapPromiseStats
where
    P: SwapPromise<M>,
{
    let mut stats = SwapPromiseStats::default();
    let mut persistent = Vec::new();
    for promise in promises.iter_mut() {
        stats.visited += 1;
        if promise
            .as_mut()
            .expect("non-null swap promise")
            .DidNotSwap(reason, timestamp)
            == DidNotSwapAction::KEEP_ACTIVE
        {
            stats.kept_active += 1;
            persistent.push(promise.take());
        } else {
            stats.broken += 1;
        }
    }
    *promises = persistent;
    stats
}

pub fn BreakSwapPromises<P, M: ?Sized>(
    regular: &mut Vec<Option<Box<P>>>,
    pinned: &mut Vec<Option<Box<P>>>,
    reason: DidNotSwapReason,
    timestamp: TimeTicks,
) -> SwapPromiseStats
where
    P: SwapPromise<M>,
{
    let mut trace = browser_tracing::span("compositor.frame", "Compositor.Frame.BreakSwapPromises");
    let regular_stats = BreakSwapPromiseList::<P, M>(regular, reason, timestamp);
    let pinned_stats = BreakSwapPromiseList::<P, M>(pinned, reason, timestamp);
    let stats = SwapPromiseStats {
        visited: regular_stats.visited + pinned_stats.visited,
        kept_active: regular_stats.kept_active + pinned_stats.kept_active,
        completed: 0,
        broken: regular_stats.broken + pinned_stats.broken,
    };
    trace.set("visited", stats.visited as f64);
    trace.set("kept_active", stats.kept_active as f64);
    trace.set("broken", stats.broken as f64);
    stats
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestPromise {
        keep: bool,
        will_swap: bool,
        did_swap: bool,
    }

    impl SwapPromise<()> for TestPromise {
        fn DidActivate(&mut self) {}
        fn WillSwap(&mut self, _: &mut ()) {
            self.will_swap = true;
        }
        fn DidSwap(&mut self) {
            self.did_swap = true;
        }
        fn DidNotSwap(&mut self, _: DidNotSwapReason, _: TimeTicks) -> DidNotSwapAction {
            if self.keep {
                DidNotSwapAction::KEEP_ACTIVE
            } else {
                DidNotSwapAction::BREAK_PROMISE
            }
        }
        fn GetTraceId(&self) -> i64 {
            0
        }
    }

    #[test]
    fn pass_keeps_only_active_promises_before_incoming_promises() {
        let mut current = vec![
            Some(Box::new(TestPromise {
                keep: false,
                ..TestPromise::default()
            })),
            Some(Box::new(TestPromise {
                keep: true,
                ..TestPromise::default()
            })),
        ];
        let incoming = vec![Box::new(TestPromise::default())];
        let stats =
            PassSwapPromises::<TestPromise, ()>(&mut current, incoming, TimeTicks::default());
        assert_eq!(stats.visited, 2);
        assert_eq!(stats.kept_active, 1);
        assert_eq!(stats.broken, 1);
        assert_eq!(current.len(), 2);
        assert!(!current[0].as_ref().unwrap().keep);
        assert!(current[1].as_ref().unwrap().keep);
    }

    #[test]
    fn break_preserves_keep_active_slots_in_each_queue() {
        let make = |keep| {
            Some(Box::new(TestPromise {
                keep,
                ..TestPromise::default()
            }))
        };
        let mut regular = vec![make(false), make(true)];
        let mut pinned = vec![make(true)];
        let stats = BreakSwapPromises::<TestPromise, ()>(
            &mut regular,
            &mut pinned,
            DidNotSwapReason::ACTIVATION_FAILS,
            TimeTicks::default(),
        );
        assert_eq!(stats.kept_active, 2);
        assert_eq!(stats.broken, 1);
        assert_eq!(regular.len(), 1);
        assert_eq!(pinned.len(), 1);
    }
}
