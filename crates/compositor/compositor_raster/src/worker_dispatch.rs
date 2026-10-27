//! Raster worker-lane dispatch and task-graph execution instrumentation.
//!
//! Executors remain injected by the compositor boundary. The raster package
//! owns how many lanes are dispatched and the spans around graph scheduling
//! and worker draining, so queueing and execution time are attributable here.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RasterWorkerDispatchStats {
    pub workers_dispatched: usize,
}

pub fn DispatchRasterWorkerLanes(
    worker_limit: usize,
    mut dispatch: impl FnMut(usize),
) -> RasterWorkerDispatchStats {
    let mut trace = browser_tracing::span("compositor.raster", "Compositor.Raster.DispatchWorkers");
    for lane in 0..worker_limit {
        dispatch(lane);
    }
    trace.set("workers", worker_limit as f64);
    RasterWorkerDispatchStats {
        workers_dispatched: worker_limit,
    }
}

pub fn RunRasterWorkerUntilIdle<R>(run: impl FnOnce() -> R) -> R {
    let _trace = browser_tracing::span("compositor.raster", "Compositor.Raster.RunUntilIdle");
    run()
}

pub fn ScheduleRasterTaskGraph<R>(nodes: usize, edges: usize, schedule: impl FnOnce() -> R) -> R {
    let mut trace = browser_tracing::span("compositor.raster", "Compositor.Raster.ScheduleGraph");
    trace.set("nodes", nodes as f64);
    trace.set("edges", edges as f64);
    schedule()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispatches_exactly_the_configured_worker_lanes_in_order() {
        let mut lanes = Vec::new();
        let stats = DispatchRasterWorkerLanes(3, |lane| lanes.push(lane));
        assert_eq!(lanes, vec![0, 1, 2]);
        assert_eq!(stats.workers_dispatched, 3);
    }

    #[test]
    fn zero_worker_limit_dispatches_no_work() {
        let mut called = false;
        let stats = DispatchRasterWorkerLanes(0, |_| called = true);
        assert!(!called);
        assert_eq!(stats.workers_dispatched, 0);
    }

    #[test]
    fn execution_wrappers_preserve_results() {
        assert_eq!(RunRasterWorkerUntilIdle(|| 17), 17);
        assert_eq!(ScheduleRasterTaskGraph(4, 3, || 23), 23);
    }
}
